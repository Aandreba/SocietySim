use std::time::Duration;
use crate::{sync::{Fence}, Result, error::Error, device::Device};
use self::consumer::{EventConsumer, Map};
use super::{ContextRef, Context};

pub mod consumer;
flat_mod! { r#async }

#[derive(Debug)]
pub struct Event<C: ContextRef, N> {
    pub(crate) fence: Fence<C>,
    pub(crate) c: N
}

impl<C: ContextRef, N: EventConsumer> Event<C, N> {
    #[inline]
    pub fn new (fence: Fence<C>, f: N) -> Self {
        return Self {
            fence,
            c: f
        }
    }

    #[inline]
    pub fn context (&self) -> &Context {
        return self.fence.context()
    }

    #[inline]
    pub fn device (&self) -> &Device {
        return self.fence.device()
    }

    #[inline]
    pub fn wait (self) -> Result<N::Output> {
        self.fence.wait()?;
        return Ok(self.c.consume())
    }

    #[inline]
    pub fn wait_timeout (self, timeout: Duration) -> ::core::result::Result<N::Output, EventTimeoutError<C, N>> {
        if self.fence.wait_timeout(timeout)? {
            return Ok(self.c.consume())
        }
        return Err(EventTimeoutError::Timeout(self))
    }
}

impl<C: ContextRef, N: EventConsumer> Event<C, N> {
    #[inline]
    pub fn replace<F: EventConsumer> (self, f: F) -> (Event<C, F>, N) {
        return (
            Event {
                fence: self.fence,
                c: f
            },
            self.c
        )
    }

    #[inline]
    pub fn map<T, F: FnOnce(N::Output) -> T> (self, f: F) -> Event<C, Map<N, F>> {
        return Event {
            fence: self.fence,
            c: Map {
                f: self.c,
                u: f,
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum EventTimeoutError<C: ContextRef, F> {
    #[error("Wait timed out")]
    Timeout (Event<C, F>),
    #[error("{0}")]
    Error (#[from] Error)
}
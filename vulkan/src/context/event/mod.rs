use std::time::Duration;
use crate::{sync::{Fence}, Result, error::Error};
use self::consumer::EventConsumer;
use super::ContextRef;

pub mod consumer;
flat_mod! { r#async }

#[derive(Debug)]
pub struct Event<C: ContextRef, F> {
    pub(crate) fence: Fence<C>,
    pub(crate) f: F
}

impl<C: ContextRef, F: EventConsumer> Event<C, F> {
    #[inline]
    pub fn new (fence: Fence<C>, f: F) -> Self {
        return Self {
            fence,
            f
        }
    }

    #[inline]
    pub fn wait (self) -> Result<F::Output> {
        self.fence.wait()?;
        return Ok(self.f.consume())
    }

    #[inline]
    pub fn wait_timeout (self, timeout: Duration) -> ::core::result::Result<F::Output, EventTimeoutError<C, F>> {
        if self.fence.wait_timeout(timeout)? {
            return Ok(self.f.consume())
        }
        return Err(EventTimeoutError::Timeout(self))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum EventTimeoutError<C: ContextRef, F> {
    #[error("Wait timed out")]
    Timeout (Event<C, F>),
    #[error("{0}")]
    Error (#[from] Error)
}
use std::{time::Duration, mem::ManuallyDrop};
use elor::Either;
use serde::__private::de;

use crate::{sync::{Fence}, Result, error::Error, device::Device, Entry, utils::usize_to_u32};
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
    pub fn id (&self) -> u64 {
        return self.fence.id()
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
    pub unsafe fn consume_unchecked (self) -> N::Output {
        return self.c.consume()
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

    #[inline]
    pub fn join_all<I: IntoIterator<Item = Self>> (iter: I) -> Result<impl Iterator<Item = N::Output>> {
        #[inline]
        fn wait_for_fences(device: u64, fences: &[u64], nanos: u64) -> vk::Result {
            return (Entry::get().wait_for_fences)(
                device,
                usize_to_u32(fences.len()),
                fences.as_ptr(),
                vk::TRUE,
                nanos,
            );
        }

        #[inline]
        fn drop_fences (device: u64, fences: &[u64]) {
            for fence in fences {
                (Entry::get().destroy_fence)(
                    device,
                    *fence,
                    core::ptr::null()
                )
            }
        }

        let iter = iter.into_iter();
        let capacity = match iter.size_hint() {
            (_, Some(x)) => x,
            (x, _) => x
        };

        let mut device = None;
        let mut fences = Vec::with_capacity(capacity);
        let mut consumers = Vec::with_capacity(capacity);

        for event in iter {
            if device.is_none() { device = Some(event.device().id()) }
            debug_assert_eq!(device, Some(event.device().id()));

            fences.push(ManuallyDrop::new(event.fence).id());
            consumers.push(event.c);
        }

        if let Some(device) = device {
            loop {
                match wait_for_fences(device, &fences, u64::MAX) {
                    vk::SUCCESS => {
                        drop_fences(device, &fences);
                        return Ok(Either::Left(consumers.into_iter().map(N::consume)).into_same_iter())
                    },
                    vk::TIMEOUT => {},
                    e => {
                        drop_fences(device, &fences);
                        return Err(e.into())
                    }
                }
            }
        } else {
            return Ok(Either::Right(core::iter::empty()).into_same_iter())
        }
        
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
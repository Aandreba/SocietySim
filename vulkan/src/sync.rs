use futures::Future;
use pin_project::pin_project;

use crate::{context::ContextRef, device::Device, r#async::FenceRuntimeHandle, Entry, Result};
use std::{
    num::NonZeroU64,
    ops::Deref,
    ptr::{addr_of, addr_of_mut},
    sync::{atomic::AtomicU8, Arc},
    time::Duration, future::IntoFuture, task::Poll,
};

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Fence<C: ?Sized + ContextRef> {
    inner: NonZeroU64,
    context: C,
}

impl<C: ContextRef> Fence<C> {
    /// Kinda arbitrary, may change in the future
    pub const MAX_BUDGET: Duration = Duration::from_millis(5);

    pub fn new(context: C, flags: FenceFlags) -> Result<Self> {
        let info = vk::FenceCreateInfo {
            sType: vk::STRUCTURE_TYPE_FENCE_CREATE_INFO,
            pNext: core::ptr::null(),
            flags: flags.bits(),
        };

        let mut result = 0;
        tri! {
            (Entry::get().create_fence)(
                context.device().id(),
                addr_of!(info),
                core::ptr::null(),
                addr_of_mut!(result)
            )
        }

        if let Some(inner) = NonZeroU64::new(result) {
            return Ok(Self { inner, context });
        }
        return Err(vk::ERROR_UNKNOWN.into());
    }

    #[inline]
    pub fn id(&self) -> u64 {
        return self.inner.get();
    }

    #[inline]
    pub fn device(&self) -> &Device {
        return self.context.device();
    }

    #[inline]
    pub fn status(&self) -> Result<bool> {
        return match (Entry::get().get_fence_status)(self.device().id(), self.id()) {
            vk::SUCCESS => Ok(true),
            vk::NOT_READY => Ok(false),
            e => Err(e.into()),
        };
    }

    #[inline]
    pub fn is_signaled(&self) -> bool {
        matches!(self.status(), Ok(true))
    }

    #[inline]
    pub fn is_unsignaled(&self) -> bool {
        matches!(self.status(), Ok(false))
    }

    #[inline]
    pub fn reset(&mut self) -> Result<()> {
        tri! {
            (Entry::get().reset_fences)(
                self.device().id(),
                1,
                addr_of!(self.inner).cast()
            )
        }
        return Ok(());
    }

    #[inline]
    pub fn wait(&self) -> Result<()> {
        loop {
            match self.wait_for_fences(u64::MAX) {
                vk::SUCCESS => return Ok(()),
                vk::TIMEOUT => {}
                e => return Err(e.into()),
            }
        }
    }

    pub fn wait_timeout(&self, timeout: Duration) -> Result<bool> {
        const LIMIT: u128 = u64::MAX as u128;

        let nanos = timeout.as_nanos();
        let div = nanos / LIMIT;
        let rem = (nanos % LIMIT) as u64; // [0, u64::MAX)

        for _ in 0..div {
            match self.wait_for_fences(u64::MAX) {
                vk::SUCCESS => return Ok(true),
                vk::TIMEOUT => {}
                e => return Err(e.into()),
            }
        }

        if rem > 0 {
            return match self.wait_for_fences(rem) {
                vk::SUCCESS => Ok(true),
                vk::TIMEOUT => Ok(false),
                e => Err(e.into()),
            };
        }

        return Ok(false);
    }

    #[inline]
    pub fn wait_async<'a>(&'a self, handle: FenceRuntimeHandle<'a>) -> FenceWait where C: 'a + Sync {
        self.wait_async_with_budget(FenceRuntimeHandle::MAX_BUDGET, handle)
    }

    #[inline]
    pub fn wait_async_with_budget<'a>(
        &'a self,
        budget: Duration,
        handle: FenceRuntimeHandle<'a>,
    ) -> FenceWait
    where
        C: 'a + Sync,
    {
        let (result, flag) = handle.push(self, budget);
        return FenceWait { flag, result };
    }

    #[inline]
    fn wait_for_fences(&self, nanos: u64) -> vk::Result {
        return (Entry::get().wait_for_fences)(
            self.device().id(),
            1,
            addr_of!(self.inner).cast(),
            vk::TRUE,
            nanos,
        );
    }
}

impl<D: ?Sized + ContextRef> Drop for Fence<D> {
    #[inline]
    fn drop(&mut self) {
        (Entry::get().destroy_fence)(
            self.context.device().id(),
            self.inner.get(),
            core::ptr::null(),
        )
    }
}

bitflags::bitflags! {
    #[repr(transparent)]
    pub struct FenceFlags: vk::FenceCreateFlagBits {
        const SIGNALED = vk::FENCE_CREATE_SIGNALED_BIT;
    }
}

#[derive(Debug, Clone)]
#[pin_project]
pub struct FenceWait {
    #[pin]
    flag: utils_atomics::flag::mpmc::AsyncSubscribe,
    result: Arc<vk::Result>,
}

impl Future for FenceWait {
    type Output = Result<()>;

    #[inline]
    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let this = self.project();

        if this.flag.poll(cx).is_ready() {
            let result: vk::Result = *(&this.result as &i32);
            return Poll::Ready(match result {
                vk::SUCCESS => Ok(()),
                e => Err(e.into())
            })
        }

        return Poll::Pending
    }
}

// impl<C: ContextRef> Drop for FenceWait<'_, C> {
//     #[inline]
//     fn drop(&mut self) {
//         let mut guard = match self.abort.lock() {
//             Ok(x) => x,
//             Err(e) => e.into_inner()
//         };
//         *guard = true
//     }
// }

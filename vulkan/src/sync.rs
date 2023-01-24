use crate::{context::{ContextRef, Context}, device::Device, Entry, Result};
use std::{
    num::NonZeroU64,
    ptr::{addr_of, addr_of_mut},
    time::Duration, ffi::c_void,
};

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Fence<C: ContextRef> {
    pub(crate) inner: NonZeroU64,
    pub(crate) context: C,
}

impl<C: ContextRef> Fence<C> {
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
    pub fn context(&self) -> &Context {
        return &self.context
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

    #[inline]
    pub fn wait_nanos (&self, nanos: u64) -> Result<bool> {
        return match self.wait_for_fences(nanos) {
            vk::SUCCESS => Ok(true),
            vk::TIMEOUT => Ok(false),
            e => Err(e.into()),
        }
    }

    #[inline]
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

        return match self.wait_for_fences(rem) {
            vk::SUCCESS => Ok(true),
            vk::TIMEOUT => Ok(false),
            e => Err(e.into()),
        }
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

impl<D: ContextRef> Drop for Fence<D> {
    #[inline]
    fn drop(&mut self) {
        (Entry::get().destroy_fence)(
            self.device().id(),
            self.id(),
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
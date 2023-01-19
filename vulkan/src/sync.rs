use crate::{
    device::{Device, DeviceRef},
    pool::CommandPool,
    queue::Queue,
    Entry, Result,
};
use std::{
    num::NonZeroU64,
    ptr::{addr_of, addr_of_mut},
    time::Duration,
};

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Fence<D: DeviceRef> {
    inner: NonZeroU64,
    parent: D,
}

impl<D: DeviceRef> Fence<D> {
    pub fn new(parent: D, flags: FenceFlags) -> Result<Self> {
        let info = vk::FenceCreateInfo {
            sType: vk::STRUCTURE_TYPE_FENCE_CREATE_INFO,
            pNext: core::ptr::null(),
            flags: flags.bits(),
        };

        let mut result = 0;
        tri! {
            (Entry::get().create_fence)(
                parent.id(),
                addr_of!(info),
                core::ptr::null(),
                addr_of_mut!(result)
            )
        }

        if let Some(inner) = NonZeroU64::new(result) {
            return Ok(Self { inner, parent });
        }
        return Err(vk::ERROR_UNKNOWN.into());
    }

    #[inline]
    pub fn id(&self) -> u64 {
        return self.inner.get();
    }

    #[inline]
    pub fn device(&self) -> &Device {
        return &self.parent;
    }

    #[inline]
    pub fn owned_device(&self) -> D
    where
        D: Clone,
    {
        return self.parent.clone();
    }

    #[inline]
    pub fn status(&self) -> Result<bool> {
        return match (Entry::get().get_fence_status)(self.device().id(), self.id()) {
            vk::SUCCESS => Ok(true),
            vk::NOT_READY => Ok(false),
            e => Err(e.into())
        }
    }

    #[inline]
    pub fn is_signaled (&self) -> bool {
        matches!(self.status(), Ok(true))
    }

    #[inline]
    pub fn is_unsignaled (&self) -> bool {
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
    pub fn bind_to<'b, P: DeviceRef, S: DeviceRef>(
        &mut self,
        pool: &'b mut CommandPool<P>,
        queue: &mut Queue,
        semaphores: Option<&'b [Semaphore<S>]>,
    ) -> Result<()> {
        queue
            .submitter(Some(self))
            .add(pool, 0..1, semaphores)
            .submit()
    }

    #[inline]
    pub fn wait(&self, timeout: Option<Duration>) -> Result<bool> {
        let timeout = match timeout {
            #[cfg(debug_assertions)]
            Some(x) => u64::try_from(x.as_nanos()).unwrap(),
            #[cfg(not(debug_assertions))]
            Some(x) => x.as_nanos() as u64,
            None => u64::MAX,
        };

        return match (Entry::get().wait_for_fences)(
            self.parent.id(),
            1,
            addr_of!(self.inner).cast(),
            vk::TRUE,
            timeout,
        ) {
            vk::SUCCESS => Ok(true),
            vk::TIMEOUT => Ok(false),
            e => Err(e.into()),
        };
    }

    #[inline]
    pub fn bind_and_wait<'a, P: DeviceRef, S: DeviceRef>(
        &mut self,
        pool: &'a mut CommandPool<P>,
        queue: &mut Queue,
        semaphores: Option<&'a [Semaphore<S>]>,
        timeout: Option<Duration>,
    ) -> Result<bool> {
        self.bind_to(pool, queue, semaphores)?;
        return self.wait(timeout);
    }
}

impl<D: DeviceRef> Drop for Fence<D> {
    #[inline]
    fn drop(&mut self) {
        (Entry::get().destroy_fence)(self.parent.id(), self.id(), core::ptr::null())
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Semaphore<D: DeviceRef> {
    inner: NonZeroU64,
    parent: D,
}

impl<D: DeviceRef> Semaphore<D> {
    #[inline]
    pub fn new(parent: D) -> Result<Self> {
        let info = vk::SemaphoreCreateInfo {
            sType: vk::STRUCTURE_TYPE_SEMAPHORE_CREATE_INFO,
            pNext: core::ptr::null(),
            flags: 0,
        };

        let mut result = 0;
        tri! {
            (Entry::get().create_semaphore)(
                parent.id(),
                addr_of!(info),
                core::ptr::null(),
                addr_of_mut!(result)
            )
        }

        if let Some(inner) = NonZeroU64::new(result) {
            return Ok(Self { inner, parent });
        }
        return Err(vk::ERROR_UNKNOWN.into());
    }

    #[inline]
    pub fn id(&self) -> u64 {
        return self.inner.get();
    }
}

impl<D: DeviceRef> Drop for Semaphore<D> {
    #[inline]
    fn drop(&mut self) {
        (Entry::get().destroy_semaphore)(self.parent.id(), self.id(), core::ptr::null())
    }
}

bitflags::bitflags! {
    #[repr(transparent)]
    pub struct FenceFlags: vk::FenceCreateFlagBits {
        const SIGNALED = vk::FENCE_CREATE_SIGNALED_BIT;
    }
}

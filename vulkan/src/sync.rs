use crate::{
    device::{Device},
    Entry, Result, context::ContextRef,
};
use std::{
    num::NonZeroU64,
    ptr::{addr_of, addr_of_mut},
    time::Duration,
};

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Fence<C: ContextRef> {
    inner: NonZeroU64,
    context: C,
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
    pub fn device(&self) -> &Device {
        return self.context.device();
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

    // #[inline]
    // pub fn bind_to<'b, P: ContextRef, S: ContextRef>(
    //     &mut self,
    //     pool: &'b mut CommandPool<P>,
    //     queue: &mut Queue,
    //     semaphores: Option<&'b [Semaphore<S>]>,
    // ) -> Result<()> {
    //     queue
    //         .submitter(Some(self))
    //         .add(pool, 0..1, semaphores)
    //         .submit()
    // }

    #[inline]
    pub fn wait(&self, timeout: Option<Duration>) -> Result<bool> {
        #[inline]
        fn wait_for_fences (this: &NonZeroU64, device: &Device, nanos: u64) -> vk::Result {
            return (Entry::get().wait_for_fences)(
                device.id(),
                1,
                (this as *const NonZeroU64).cast::<u64>(),
                vk::TRUE,
                nanos,
            );
        }

        if let Some(timeout) = timeout {
            const LIMIT: u128 = u64::MAX as u128;

            let nanos = timeout.as_nanos();
            let div = nanos / LIMIT;
            let rem = (nanos % LIMIT) as u64; // [0, u64::MAX)

            for _ in 0..div {
                match wait_for_fences(&self.inner, self.device(), rem) {
                    vk::SUCCESS => return Ok(true),
                    vk::TIMEOUT => {},
                    e => return Err(e.into()),
                }
            }

            if rem > 0 {
                return match wait_for_fences(&self.inner, self.device(), rem) {
                    vk::SUCCESS => Ok(true),
                    vk::TIMEOUT => Ok(false),
                    e => Err(e.into()),
                }
            }
            return Ok(false)
        } else {
            loop {
                match wait_for_fences(&self.inner, self.device(), u64::MAX) {
                    vk::SUCCESS => return Ok(true),
                    vk::TIMEOUT => {},
                    e => return Err(e.into()),
                }
            }
        }
    }

    // #[inline]
    // pub fn bind_and_wait<'a, P: ContextRef, S: ContextRef>(
    //     &mut self,
    //     pool: &'a mut CommandPool<P>,
    //     queue: &mut Queue,
    //     semaphores: Option<&'a [Semaphore<S>]>,
    //     timeout: Option<Duration>,
    // ) -> Result<bool> {
    //     self.bind_to(pool, queue, semaphores)?;
    //     return self.wait(timeout);
    // }
}

impl<D: ContextRef> Drop for Fence<D> {
    #[inline]
    fn drop(&mut self) {
        (Entry::get().destroy_fence)(self.parent.id(), self.id(), core::ptr::null())
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Semaphore<D: ContextRef> {
    inner: NonZeroU64,
    parent: D,
}

impl<D: ContextRef> Semaphore<D> {
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

impl<D: ContextRef> Drop for Semaphore<D> {
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

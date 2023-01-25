use std::{sync::{MutexGuard, TryLockError}, num::NonZeroU64, ptr::{addr_of, NonNull}, pin::Pin, mem::ManuallyDrop};
use crate::{Entry, Result, sync::{Fence, FenceFlags}};
use super::{QueueFamily, ContextRef};
flat_mod! { compute, transfer }

#[derive(Debug)]
pub struct Command<C: ContextRef> {
    ctx: Pin<C>,
    family: NonNull<QueueFamily>,
    pool_buffer: MutexGuard<'static, [NonZeroU64; 2]>,
}

impl<C: ContextRef> Command<C> {
    pub(crate) unsafe fn new (ctx: Pin<C>, family: *const QueueFamily, pool_buffer: MutexGuard<'static, [NonZeroU64; 2]>) -> Result<Self> {
        let info = vk::CommandBufferBeginInfo {
            sType: vk::STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO,
            pNext: core::ptr::null(),
            flags: 0,
            pInheritanceInfo: core::ptr::null(), // todo
        };

        tri! {
            (Entry::get().begin_command_buffer)(pool_buffer[1].get(), addr_of!(info))
        }

        return Ok(Self {
            ctx,
            pool_buffer: core::mem::transmute(pool_buffer),
            family: NonNull::new_unchecked(family.cast_mut())
        })
    }

    #[inline]
    pub fn pool (&self) -> vk::CommandPool {
        return self.pool_buffer[0].get()
    }
    
    #[inline]
    pub fn buffer (&self) -> vk::CommandBuffer {
        return self.pool_buffer[1].get()
    }

    #[inline]
    pub fn submit (self) -> Result<Fence<Pin<C>>> {
        let this = ManuallyDrop::new(self);
        let ctx = unsafe { core::ptr::read(&this.ctx) };
        let family = this.family;
        let pool_buffer = unsafe { core::ptr::read(&this.pool_buffer) };
        drop(this); // does not invoke the inner's destructor

        let fence = Fence::new(ctx, FenceFlags::empty())?;
        let buffer = pool_buffer[1].get();
        tri! {
            (Entry::get().end_command_buffer)(buffer)
        }

        let info = vk::SubmitInfo {
            sType: vk::STRUCTURE_TYPE_SUBMIT_INFO,
            pNext: core::ptr::null(),
            waitSemaphoreCount: 0, // todo
            pWaitSemaphores: core::ptr::null(), // todo
            pWaitDstStageMask: core::ptr::null(), // todo
            commandBufferCount: 1,
            pCommandBuffers: addr_of!(buffer),
            signalSemaphoreCount: 0, // todo
            pSignalSemaphores: core::ptr::null(), // todo
        };

        let queue = 'outer: loop {
            for queue in unsafe { family.as_ref() }.queues.iter() {
                match queue.try_lock() {
                    Ok(x) => break 'outer x,
                    Err(TryLockError::Poisoned(e)) => break 'outer e.into_inner(),
                    Err(_) => {}
                }
            }
            std::thread::yield_now();
        };

        tri! {
            (Entry::get().queue_submit)(
                queue.get(),
                1,
                addr_of!(info),
                fence.id()
            )
        }
        
        return Ok(fence)
    }
}

impl<C: ContextRef> Drop for Command<C> {
    #[inline]
    fn drop(&mut self) {
        let v = (Entry::get().end_command_buffer)(self.buffer());
        debug_assert_eq!(v, vk::SUCCESS)
    }
}
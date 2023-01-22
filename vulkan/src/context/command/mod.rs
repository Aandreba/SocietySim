use std::{sync::{MutexGuard, TryLockError}, num::NonZeroU64, ptr::addr_of};
use crate::{Entry, Result, error::Error};
use super::QueueFamily;

flat_mod! { compute, transfer }
pub type CommandResult<'a, T> = ::core::result::Result<T, CommandError<'a>>;

pub struct CommandError<'a> {
    command: Command<'a>,
    error: Error
}

impl<'a> CommandError<'a> {
    #[inline]
    pub fn into_inner (self) -> Command<'a> {
        self.command
    }
}

impl Into<Error> for CommandError<'_> {
    #[inline]
    fn into(self) -> Error {
        self.error
    }
}

#[derive(Debug)]
pub struct Command<'a> {
    family: &'a QueueFamily,
    pool_buffer: MutexGuard<'a, [NonZeroU64; 2]>,
}

impl<'a> Command<'a> {
    pub(crate) fn new (family: &'a QueueFamily, pool_buffer: MutexGuard<'a, [NonZeroU64; 2]>) -> Result<Self> {
        let info = vk::CommandBufferBeginInfo {
            sType: vk::STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO,
            pNext: core::ptr::null(),
            flags: vk::COMMAND_BUFFER_USAGE_ONE_TIME_SUBMIT_BIT, // todo
            pInheritanceInfo: core::ptr::null(), // todo
        };

        tri! {
            (Entry::get().begin_command_buffer)(pool_buffer[1].get(), addr_of!(info))
        }

        return Ok(Self { pool_buffer, family })
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
    pub fn submit (self) -> Result<()> {
        let family = self.family;
        //let pool = self.pool();
        let buffer = self.buffer();
        drop(self);
        
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
            for queue in family.queues.iter() {
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
                vk::NULL_HANDLE // todo
            )
        }

        return Ok(())
    }
}

impl Drop for Command<'_> {
    #[inline]
    fn drop(&mut self) {
        let v = (Entry::get().end_command_buffer)(self.buffer());
        debug_assert_eq!(v, vk::SUCCESS)
    }
}
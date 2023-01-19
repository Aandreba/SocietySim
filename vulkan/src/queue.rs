use std::{num::NonZeroU64, ptr::{addr_of, addr_of_mut}, marker::PhantomData, sync::{RwLock, RwLockReadGuard, LockResult}, time::Duration, slice::SliceIndex};
use crate::{device::{DeviceRef, Device}, Entry, Result, utils::usize_to_u32, pool::{CommandPool}, sync::{Fence, Semaphore}};

#[derive(Debug, PartialEq, Hash)]
pub struct Queue {
    pub(super) inner: NonZeroU64,
    pub(super) index: u32,
    //pub(super) parent: &'a Device
}

impl Queue {
    #[inline]
    pub fn id (&self) -> u64 {
        return self.inner.get()
    }

    #[inline]
    pub fn submitter<'a, F: DeviceRef, P: DeviceRef, S: DeviceRef> (&'a mut self, fence: Option<&'a mut Fence<F>>) -> SubmitBuilder<'a, F, P, S> {
        return SubmitBuilder {
            queue: self,
            fence,
            locks: Vec::with_capacity(1),
            semaphores: Vec::with_capacity(1),
            submits: Vec::with_capacity(1),
            _phtm: PhantomData,
        }
    }
}

pub struct SubmitBuilder<'a, F: DeviceRef, P: DeviceRef, S: DeviceRef = &'a Device> {
    queue: &'a mut Queue,
    fence: Option<&'a mut Fence<F>>,
    submits: Vec<vk::SubmitInfo>,
    locks: Vec<Vec<LockResult<RwLockReadGuard<'a, ()>>>>,
    semaphores: Vec<Vec<vk::Semaphore>>,
    _phtm: PhantomData<(&'a [Semaphore<S>], &'a CommandPool<P>)>
}

impl<'a, F: DeviceRef, S: DeviceRef, P: DeviceRef> SubmitBuilder<'a, F, P, S> {
    #[inline]
    pub fn add<B: Clone + SliceIndex<[RwLock<()>], Output = [RwLock<()>]> + SliceIndex<[vk::CommandBuffer], Output = [vk::CommandBuffer]>> (mut self, pool: &'a CommandPool<P>, buffers: B, semaphores: Option<&'a [Semaphore<S>]>) -> Self {
        let semaphores = semaphores.into_iter().flatten().map(Semaphore::id).collect::<Vec<_>>();
        let locks = pool.locks[buffers.clone()].iter().map(RwLock::read).collect::<Vec<_>>();
        let buffers = &pool.buffers[buffers];

        self.submits.push(vk::SubmitInfo {
            sType: vk::STRUCTURE_TYPE_SUBMIT_INFO,
            pNext: core::ptr::null(),
            waitSemaphoreCount: usize_to_u32(semaphores.len()),
            pWaitSemaphores: semaphores.as_ptr(),
            pWaitDstStageMask: core::ptr::null(),
            commandBufferCount: usize_to_u32(buffers.len()),
            pCommandBuffers: buffers.as_ptr(),
            signalSemaphoreCount: 0, // todo
            pSignalSemaphores: core::ptr::null(), // todo
        });

        self.locks.push(locks);
        self.semaphores.push(semaphores);
        return self
    }

    #[inline]
    pub fn submit (self) -> Result<()> {
        tri! {
            (Entry::get().queue_submit)(
                self.queue.id(),
                usize_to_u32(self.submits.len()),
                self.submits.as_ptr(),
                self.fence.map_or(vk::NULL_HANDLE, |x| x.id())
            )
        }
        return Ok(())
    }
}

// bitflags::bitflags! {
//     #[repr(transparent)]
//     pub struct SemaphoreFlags: vk::SemaphoreCreateFlagBits {
//     }
// }
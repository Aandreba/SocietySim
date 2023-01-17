use std::{num::NonZeroU64, ptr::{addr_of, addr_of_mut}, marker::PhantomData, ops::Range, sync::{RwLock, RwLockReadGuard, LockResult}, time::Duration, slice::SliceIndex};
use crate::{device::Device, Entry, Result, utils::usize_to_u32, pool::{CommandPool}};

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
    pub fn submitter<'a, 'b> (&'b mut self, fence: Option<&'b mut Fence<'a>>) -> SubmitBuilder<'a, 'b> {
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

pub struct SubmitBuilder<'a, 'b> {
    queue: &'b mut Queue,
    fence: Option<&'b mut Fence<'a>>,
    submits: Vec<vk::SubmitInfo>,
    locks: Vec<Vec<LockResult<RwLockReadGuard<'b, ()>>>>,
    semaphores: Vec<Vec<vk::Semaphore>>,
    _phtm: PhantomData<(&'b [Semaphore<'a>], &'b CommandPool<'a>)>
}

impl<'a, 'b> SubmitBuilder<'a, 'b> {
    #[inline]
    pub fn add<S: Clone + SliceIndex<[RwLock<()>], Output = [RwLock<()>]> + SliceIndex<[vk::CommandBuffer], Output = [vk::CommandBuffer]>> (mut self, pool: &'b CommandPool, buffers: S, semaphores: Option<&'b [Semaphore<'a>]>) -> Self {
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

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Fence<'a> {
    inner: NonZeroU64,
    parent: &'a Device
}

impl<'a> Fence<'a> {
    pub fn new (parent: &'a Device, flags: FenceFlags) -> Result<Self> {
        let info = vk::FenceCreateInfo {
            sType: vk::STRUCTURE_TYPE_FENCE_CREATE_INFO,
            pNext: core::ptr::null(),
            flags: flags.bits(),
        };

        let mut result = 0;
        tri!{
            (Entry::get().create_fence)(
                parent.id(),
                addr_of!(info),
                core::ptr::null(),
                addr_of_mut!(result)
            )
        }

        if let Some(inner) = NonZeroU64::new(result) {
            return Ok(Self { inner, parent })
        }
        return Err(vk::ERROR_UNKNOWN.into())
    }

    #[inline]
    pub fn id (&self) -> u64 {
        return self.inner.get()
    }

    #[inline]
    pub fn bind_to<'b> (&mut self, pool: &'b mut CommandPool, queue: &mut Queue, semaphores: Option<&'b [Semaphore<'a>]>) -> Result<()> {
        queue.submitter(Some(self))
            .add(pool, .., semaphores)
            .submit()
    }

    #[inline]
    pub fn wait (&self, timeout: Option<Duration>) -> Result<bool> {
        let timeout = match timeout {
            #[cfg(debug_assertions)]
            Some(x) => u64::try_from(x.as_nanos()).unwrap(),
            #[cfg(not(debug_assertions))]
            Some(x) => x.as_nanos() as u64,
            None => u64::MAX
        };

        return match (Entry::get().wait_for_fences)(
            self.parent.id(),
            1,
            addr_of!(self.inner).cast(),
            vk::TRUE,
            timeout
        ) {
            vk::SUCCESS => Ok(true),
            vk::TIMEOUT => Ok(false),
            e => Err(e.into())
        }
    }

    #[inline]
    pub fn bind_and_wait<'b> (&mut self, pool: &'b mut CommandPool, queue: &mut Queue, semaphores: Option<&'b [Semaphore<'a>]>, timeout: Option<Duration>) -> Result<bool> {
        self.bind_to(pool, queue, semaphores)?;
        return self.wait(timeout)
    }
}

impl Drop for Fence<'_> {
    #[inline]
    fn drop(&mut self) {
        (Entry::get().destroy_fence)(
            self.parent.id(),
            self.id(),
            core::ptr::null()
        )
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Semaphore<'a> {
    inner: NonZeroU64,
    parent: &'a Device
}

impl<'a> Semaphore<'a> {
    #[inline]
    pub fn new (parent: &'a Device) -> Result<Self> {
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
            return Ok(Self { inner, parent })
        }
        return Err(vk::ERROR_UNKNOWN.into())
    }

    #[inline]
    pub fn id (&self) -> u64 {
        return self.inner.get()
    }
}

impl Drop for Semaphore<'_> {
    #[inline]
    fn drop(&mut self) {
        (Entry::get().destroy_semaphore)(
            self.parent.id(),
            self.id(),
            core::ptr::null()
        )
    }
}

bitflags::bitflags! {
    #[repr(transparent)]
    pub struct FenceFlags: vk::FenceCreateFlagBits {
        const SIGNALED = vk::FENCE_CREATE_SIGNALED_BIT;
    }
}

// bitflags::bitflags! {
//     #[repr(transparent)]
//     pub struct SemaphoreFlags: vk::SemaphoreCreateFlagBits {
//     }
// }
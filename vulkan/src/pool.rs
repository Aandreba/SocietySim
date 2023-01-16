use std::{num::NonZeroU64, ptr::{addr_of, addr_of_mut}, ops::{Deref, Index, IndexMut}, sync::{MutexGuard, Mutex, TryLockError, PoisonError}};
use crate::{Result, Entry, physical_dev::Family, device::Device, utils::usize_to_u32};

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct CommandPool<'a> {
    inner: NonZeroU64,
    parent: &'a Device
}

impl<'a> CommandPool<'a> {
    pub fn new (parent: &'a Device, family: Family, flags: CommandPoolFlags) -> Result<Self> {
        let info = vk::CommandPoolCreateInfo {
            sType: vk::STRUCTURE_TYPE_COMMAND_POOL_CREATE_INFO,
            pNext: core::ptr::null(),
            flags: flags.bits(),
            queueFamilyIndex: family.idx(),
        };

        let mut result = 0;
        tri! {
            (Entry::get().create_command_pool)(parent.id(), addr_of!(info), core::ptr::null(), addr_of_mut!(result))
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
    pub fn device (&self) -> &Device {
        return &self.parent
    }

    #[inline]
    pub fn allocate_buffers (&mut self, capacity: u32, level: CommandBufferLevel) -> Result<CommandBuffers<'_, 'a>> {
        return CommandBuffers::new(self, capacity, level)
    }
}

impl Drop for CommandPool<'_> {
    #[inline]
    fn drop(&mut self) {
        (Entry::get().destroy_command_pool)(self.parent.id(), self.id(), core::ptr::null())
    }
}

pub struct CommandBuffers<'a, 'b> {
    inner: Box<[std::sync::Mutex<vk::CommandBuffer>]>,
    parent: &'b mut CommandPool<'a>
}

impl<'a, 'b> CommandBuffers<'a, 'b> {
    pub fn new (parent: &'b mut CommandPool<'a>, capacity: u32, level: CommandBufferLevel) -> Result<Self> {
        let info = vk::CommandBufferAllocateInfo {
            sType: vk::STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO,
            pNext: core::ptr::null(),
            commandPool: parent.id(),
            level: level as i32,
            commandBufferCount: capacity,
        };

        let mut inner = Box::<[vk::CommandBuffer]>::new_uninit_slice(capacity as usize);
        tri! {
            (Entry::get().allocate_command_buffers)(parent.device().id(), addr_of!(info), inner.as_mut_ptr().cast())
        }

        let inner = unsafe { 
            inner.assume_init().into_vec().into_iter()
                .map(Mutex::new)
                .collect::<Box<[_]>>()
        };
        return Ok(Self { inner, parent })
    }

    #[inline]
    pub fn pool (&self) -> &CommandPool<'a> {
        return &self.parent
    }

    #[inline]
    pub fn device (&self) -> &Device {
        return self.pool().device()
    }

    #[inline]
    pub fn len (&self) -> u32 {
        return usize_to_u32(self.inner.len())
    }

    #[inline]
    pub fn begin (&self, idx: u32) -> CommandBuffer<'_> {
        return match self.inner[idx as usize].lock() {
            Ok(inner) => CommandBuffer { inner: CommandBufferInner::Guard(inner) },
            Err(e) => CommandBuffer { inner: CommandBufferInner::Guard(e.into_inner()) },
        }
    }

    #[inline]
    pub fn begin_mut (&mut self, idx: u32) -> CommandBuffer<'_> {
        return match self.inner[idx as usize].get_mut() {
            Ok(inner) => CommandBuffer { inner: CommandBufferInner::Ref(inner) },
            Err(e) => CommandBuffer { inner: CommandBufferInner::Ref(e.into_inner()) },
        }
    }
    
    #[inline]
    pub fn try_begin (&self, idx: u32) -> Option<CommandBuffer<'_>> {
        return match self.inner.get(idx as usize)?.try_lock() {
            Ok(inner) => Some(CommandBuffer { inner: CommandBufferInner::Guard(inner) }),
            Err(TryLockError::Poisoned(e)) => Some(CommandBuffer { inner: CommandBufferInner::Guard(e.into_inner()) }),
            Err(_) => None
        }
    }
}

impl Drop for CommandBuffers<'_, '_> {
    #[inline]
    fn drop(&mut self) {
        (Entry::get().free_command_buffers)(
            self.device().id(),
            self.pool().id(),
            usize_to_u32(self.inner.len()),
            self.inner.as_ptr().cast()
        )
    }
}

#[derive(Debug)]
enum CommandBufferInner<'a> {
    Ref (&'a mut vk::CommandBuffer),
    Guard (MutexGuard<'a, vk::CommandBuffer>)
}

#[repr(transparent)]
#[derive(Debug)]
pub struct CommandBuffer<'a> {
    inner: CommandBufferInner<'a>
}

impl<'a> CommandBuffer<'a> {
    fn new_guard (inner: CommandBufferInner<'a>) -> Self {
        let mut this = Self { inner };
        
        return this   
    }

    #[inline]
    pub fn id (&self) -> vk::CommandBuffer {
        return match &self.inner {
            CommandBufferInner::Guard(x) => **x,
            CommandBufferInner::Ref(x) => **x
        }
    }
}

bitflags::bitflags! {
    #[repr(transparent)]
    pub struct CommandPoolFlags: vk::CommandPoolCreateFlagBits {
        /// Command buffers have a short lifetime
        const TRANSIENT = vk::COMMAND_POOL_CREATE_TRANSIENT_BIT; 
        /// Command buffers may release their memory individually
        const RESET_COMMAND_BUFFER = vk::COMMAND_POOL_CREATE_RESET_COMMAND_BUFFER_BIT;
        /// Command buffers allocated from pool are protected command buffers
        const PROTECTED = vk::COMMAND_POOL_CREATE_PROTECTED_BIT;
    }
}

#[repr(i32)]
pub enum CommandBufferLevel {
    Primary = vk::COMMAND_BUFFER_LEVEL_PRIMARY,
    Secondary = vk::COMMAND_BUFFER_LEVEL_SECONDARY,
}
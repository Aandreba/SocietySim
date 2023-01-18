use std::{num::NonZeroU64, ptr::{addr_of, addr_of_mut}, sync::{TryLockError, RwLockWriteGuard, RwLock, RwLockReadGuard}, marker::PhantomData, ops::{RangeBounds, Bound, Deref, Index}, slice::SliceIndex, ffi::c_void};
use crate::{Result, Entry, physical_dev::Family, device::{Device, DeviceRef}, utils::usize_to_u32, pipeline::Pipeline, descriptor::{DescriptorSet}, shader::ShaderStages};

#[derive(Debug)]
pub struct CommandPool<D: DeviceRef> {
    inner: NonZeroU64,
    pub(crate) locks: Box<[std::sync::RwLock<()>]>,
    pub(crate) buffers: Box<[vk::CommandBuffer]>,
    parent: D
}

impl<D: DeviceRef> CommandPool<D> {
    pub fn new (parent: D, family: Family, flags: CommandPoolFlags, capacity: u32, level: CommandBufferLevel) -> Result<Self> {
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
            let (locks, buffers) = match Self::create_buffers(inner, &parent, capacity, level) {
                Ok(x) => x,
                Err(e) => {
                    (Entry::get().destroy_command_pool)(parent.id(), inner.get(), core::ptr::null());
                    return Err(e)
                }
            };
            return Ok(Self { inner, locks, buffers, parent })
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
    pub fn get_slice<I: Clone> (&self, bounds: I) -> Vec<CommandBuffer<'_>> where
        I: SliceIndex<[RwLock<()>], Output = [RwLock<()>]>
        +  SliceIndex<[vk::CommandBuffer], Output = [vk::CommandBuffer]>,
    {
        let locks = &self.locks[bounds.clone()];
        let locks = locks.into_iter().map(RwLock::read);

        let buffers = &self.buffers[bounds];
        let buffers = buffers.into_iter();

        return buffers.zip(locks)
            .map(|(inner, lock)| match lock {
                Ok(lock) => CommandBuffer { inner: *inner, _lock: CommandBufferLock::Guard(lock) },
                Err(e) => CommandBuffer { inner: *inner, _lock: CommandBufferLock::Guard(e.into_inner()) },
            })
            .collect::<Vec<_>>();
    }

    #[inline]
    pub fn get_slice_mut<I: Clone> (&mut self, bounds: I) -> Vec<CommandBuffer<'_>> where
        I: SliceIndex<[RwLock<()>], Output = [RwLock<()>]>
        +  SliceIndex<[vk::CommandBuffer], Output = [vk::CommandBuffer]>,
    {
        let locks = &mut self.locks[bounds.clone()];
        let locks = locks.into_iter().map(RwLock::get_mut);

        let buffers = &self.buffers[bounds];
        let buffers = buffers.into_iter();

        return buffers.zip(locks)
            .map(|(inner, lock)| match lock {
                Ok(lock) => CommandBuffer { inner: *inner, _lock: CommandBufferLock::Ref(lock) },
                Err(e) => CommandBuffer { inner: *inner, _lock: CommandBufferLock::Ref(e.into_inner()) },
            })
            .collect::<Vec<_>>();
    }

    #[inline]
    pub fn begin<P: DeviceRef> (&self, idx: u32, flags: CommandBufferUsage) -> Result<Command<'_, P>> {
        return match self.locks[idx as usize].write() {
            Ok(inner) => Command::new(self.buffers[idx as usize], CommandLock::Guard(inner), flags),
            Err(e) => Command::new(self.buffers[idx as usize], CommandLock::Guard(e.into_inner()), flags),
        }
    }

    #[inline]
    pub fn begin_mut<P: DeviceRef> (&mut self, idx: u32, flags: CommandBufferUsage) -> Result<Command<'_, P>> {
        return match self.locks[idx as usize].get_mut() {
            Ok(inner) => Command::new(self.buffers[idx as usize], CommandLock::Ref(inner), flags),
            Err(e) => Command::new(self.buffers[idx as usize], CommandLock::Ref(e.into_inner()), flags),
        }
    }
    
    #[inline]
    pub fn try_begin<P: DeviceRef> (&self, idx: u32, flags: CommandBufferUsage) -> Result<Option<Command<'_, P>>> {
        return match self.locks[idx as usize].try_write() {
            Ok(inner) => Command::new(self.buffers[idx as usize], CommandLock::Guard(inner), flags).map(Some),
            Err(TryLockError::Poisoned(e)) => Command::new(self.buffers[idx as usize], CommandLock::Guard(e.into_inner()), flags).map(Some),
            Err(_) => Ok(None)
        }
    }

    fn create_buffers (parent: NonZeroU64, device: &Device, capacity: u32, level: CommandBufferLevel) -> Result<(Box<[std::sync::RwLock<()>]>, Box<[vk::CommandBuffer]>)> {
        let info = vk::CommandBufferAllocateInfo {
            sType: vk::STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO,
            pNext: core::ptr::null(),
            commandPool: parent.get(),
            level: level as i32,
            commandBufferCount: capacity,
        };

        let mut inner = Box::<[vk::CommandBuffer]>::new_uninit_slice(capacity as usize);
        tri! {
            (Entry::get().allocate_command_buffers)(device.id(), addr_of!(info), inner.as_mut_ptr().cast())
        }
        let inner = unsafe { inner.assume_init() };
        let locks = (0..inner.len()).map(|_| RwLock::new(())).collect::<Box<[_]>>();
        
        return Ok((locks, inner))
    }
}

impl<D: DeviceRef> Drop for CommandPool<D> {
    #[inline]
    fn drop(&mut self) {
        (Entry::get().free_command_buffers)(
            self.device().id(),
            self.id(),
            usize_to_u32(self.buffers.len()),
            self.buffers.as_ptr().cast()
        );
        (Entry::get().destroy_command_pool)(self.parent.id(), self.id(), core::ptr::null())
    }
}

#[derive(Debug)]
enum CommandBufferLock<'a> {
    Ref (&'a mut ()),
    Guard (RwLockReadGuard<'a, ()>)
}

#[derive(Debug)]
pub struct CommandBuffer<'a> {
    inner: vk::CommandBuffer,
    _lock: CommandBufferLock<'a>
}

impl CommandBuffer<'_> {
    #[inline]
    pub fn id (&self) -> u64 {
        return self.inner
    }
}

#[derive(Debug)]
enum CommandLock<'a> {
    Ref (&'a mut ()),
    Guard (RwLockWriteGuard<'a, ()>)
}

#[derive(Debug)]
pub struct Command<'a, P: DeviceRef> {
    inner: vk::CommandBuffer,
    pipeline: Option<&'a Pipeline<P>>,
    _lock: CommandLock<'a>
}

impl<'a, P: DeviceRef> Command<'a, P> {
    fn new (inner: vk::CommandBuffer, lock: CommandLock<'a>, flags: CommandBufferUsage) -> Result<Self> {
        let this = Self { inner, pipeline: None, _lock: lock };
        let info = vk::CommandBufferBeginInfo {
            sType: vk::STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO,
            pNext: core::ptr::null(),
            flags: flags.bits(),
            pInheritanceInfo: core::ptr::null(), // todo
        };

        tri! {
            (Entry::get().begin_command_buffer)(this.id(), addr_of!(info))
        }

        return Ok(this)
    }

    #[inline]
    pub fn id (&self) -> vk::CommandBuffer {
        return self.inner
    }

    #[inline]
    pub fn push_contant<T: Copy> (&mut self, value: &T, stages: ShaderStages) -> Result<()> {
        let pipeline = self.pipeline.ok_or(vk::ERROR_NOT_PERMITTED_KHR)?;
        (Entry::get().cmd_push_constants)(
            self.id(),
            pipeline.layout(),
            stages.bits(),
            0,
            usize_to_u32(core::mem::size_of::<T>()),
            value as *const T as *const c_void
        );
        return Ok(())
    }

    #[inline]
    pub fn bind_pipeline<R: RangeBounds<usize>> (&mut self, point: PipelineBindPoint, pipeline: &'a Pipeline<P>, desc_sets: R) where [DescriptorSet]: Index<R, Output = [DescriptorSet]> {
        (Entry::get().cmd_bind_pipeline)(
            self.id(),
            point as i32,
            pipeline.id()
        );

        let first_set = match desc_sets.start_bound() {
            Bound::Excluded(x) => usize_to_u32(*x + 1),
            Bound::Included(x) => usize_to_u32(*x),
            Bound::Unbounded => 0
        };

        let descriptor_set_count = usize_to_u32(match desc_sets.end_bound() {
            Bound::Excluded(x) => *x,
            Bound::Included(x) => *x + 1,
            Bound::Unbounded => pipeline.sets().len()
        }) - first_set;

        let descriptor_sets: &[DescriptorSet] = &pipeline.sets().deref()[desc_sets];

        (Entry::get().cmd_bind_descriptor_sets)(
            self.id(),
            point as i32,
            pipeline.layout(),
            first_set,
            descriptor_set_count,
            descriptor_sets.as_ptr().cast(),
            0,
            core::ptr::null()
        );
        self.pipeline = Some(pipeline);
    }

    #[inline]
    pub fn dispatch (&mut self, x: u32, y: u32, z: u32) {
        (Entry::get().cmd_dispatch)(
            self.id(),
            x, y, z
        );
    }
}

impl<P: DeviceRef> Drop for Command<'_, P> {
    #[inline]
    fn drop(&mut self) {
        let v = (Entry::get().end_command_buffer)(self.id());
        debug_assert_eq!(v, vk::SUCCESS)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
#[non_exhaustive]
pub enum PipelineBindPoint {
    Graphics = vk::PIPELINE_BIND_POINT_GRAPHICS,
    Compute = vk::PIPELINE_BIND_POINT_COMPUTE,
    RayTracing = vk::PIPELINE_BIND_POINT_RAY_TRACING_KHR,
    SubpassShadingHuawei = vk::PIPELINE_BIND_POINT_SUBPASS_SHADING_HUAWEI,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
#[non_exhaustive]
pub enum CommandBufferLevel {
    Primary = vk::COMMAND_BUFFER_LEVEL_PRIMARY,
    Secondary = vk::COMMAND_BUFFER_LEVEL_SECONDARY,
}

bitflags::bitflags! {
    #[repr(transparent)]
    #[non_exhaustive]
    pub struct CommandPoolFlags: vk::CommandPoolCreateFlagBits {
        /// Command buffers have a short lifetime
        const TRANSIENT = vk::COMMAND_POOL_CREATE_TRANSIENT_BIT; 
        /// Command buffers may release their memory individually
        const RESET_COMMAND_BUFFER = vk::COMMAND_POOL_CREATE_RESET_COMMAND_BUFFER_BIT;
        /// Command buffers allocated from pool are protected command buffers
        const PROTECTED = vk::COMMAND_POOL_CREATE_PROTECTED_BIT;
    }
}

bitflags::bitflags! {
    #[repr(transparent)]
    #[non_exhaustive]
    pub struct CommandBufferUsage: vk::CommandBufferUsageFlagBits {
        const ONE_TIME_SUBMIT = vk::COMMAND_BUFFER_USAGE_ONE_TIME_SUBMIT_BIT;
        const RENDER_PASS_CONTINUE = vk::COMMAND_BUFFER_USAGE_RENDER_PASS_CONTINUE_BIT;
        /// Command buffer may be submitted/executed more than once simultaneously
        const SIMULTANEOUS_USE = vk::COMMAND_BUFFER_USAGE_SIMULTANEOUS_USE_BIT;
    }
}
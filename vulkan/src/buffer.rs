use std::{marker::PhantomData, num::NonZeroU64, ptr::{addr_of, addr_of_mut, NonNull}, mem::{MaybeUninit, self}, ops::{Deref, DerefMut}};
use vk::{DeviceSize, MemoryType};
use crate::{Result, Entry, device::{Device}};

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Buffer<'a, T> {
    inner: NonZeroU64,
    parent: &'a Device,
    _phtm: PhantomData<T>
}

impl<'a, T> Buffer<'a, T> {
    pub fn new_uninit (parent: &'a Device, capacity: DeviceSize, usage: UsageFlags, flags: BufferFlags) -> Result<Buffer<'a, MaybeUninit<T>>> {
        let entry = Entry::get();
        let info = vk::BufferCreateInfo {
            sType: vk::STRUCTURE_TYPE_BUFFER_CREATE_INFO,
            pNext: core::ptr::null_mut(),
            flags: flags.bits(),
            size: capacity * (core::mem::size_of::<T>() as DeviceSize),
            usage: usage.bits(),
            sharingMode: vk::SHARING_MODE_EXCLUSIVE,
            queueFamilyIndexCount: 0,
            pQueueFamilyIndices: core::ptr::null(),
        };

        let mut inner = 0;
        tri! {
            (entry.create_buffer)(parent.id(), addr_of!(info), core::ptr::null(), addr_of_mut!(inner))
        };

        if let Some(inner) = NonZeroU64::new(inner) {
            return Ok(Buffer { inner, parent, _phtm: PhantomData })
        }

        return Err(vk::ERROR_INITIALIZATION_FAILED.into())
    }

    #[inline]
    pub fn id (&self) -> u64 {
        return self.inner.get()
    }

    #[inline]
    pub fn map (&mut self) -> Result<MapGuard<'a, '_, T>> {
        let entry = Entry::get();

        let mut requirements = MaybeUninit::uninit();
        (entry.get_buffer_memory_requirements)(self.parent.id(), self.inner.get(), requirements.as_mut_ptr());
        let requirements = unsafe { requirements.assume_init() };

        let mut props = MaybeUninit::uninit();
        (entry.get_physical_device_memory_properties)(self.parent.physical().id(), props.as_mut_ptr());
        let props = unsafe { props.assume_init() };

        let mut info = None;
        for i in 0..props.memoryTypeCount {
            const FLAG: vk::MemoryPropertyFlagBits = vk::MEMORY_PROPERTY_HOST_VISIBLE_BIT | vk::MEMORY_PROPERTY_HOST_COHERENT_BIT;
            let MemoryType { propertyFlags, heapIndex } = props.memoryTypes[i as usize];

            if (propertyFlags & FLAG) != 0 {
                info = Some((props.memoryHeaps[heapIndex as usize].size, i));
                break
            }
        }

        if let Some((heap_idx, mem_type_idx)) = info {
            let info = vk::MemoryAllocateInfo {
                sType: vk::STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO,
                pNext: core::ptr::null(),
                allocationSize: requirements.size,
                memoryTypeIndex: mem_type_idx,
            };

            let mut memory = 0;
            tri! {
                (entry.allocate_memory)(self.parent.id(), addr_of!(info), core::ptr::null(), addr_of_mut!(memory))
            };

            if let Some(memory) = NonZeroU64::new(memory) {
                let ptr = 0;
                (entry.map_memory)(self.parent.id(), memory.get(), );

                let ptr = unsafe { core::slice::from_raw_parts_mut(data, requirements.size / core::mem::size_of::<T>()) };
                return Ok(MapGuard { memory, ptr, buffer: self })
            }
        }
        
        return Err(vk::ERROR_MEMORY_MAP_FAILED.into())
    }
}

impl<T> Drop for Buffer<'_, T> {
    #[inline]
    fn drop(&mut self) {
        (Entry::get().destroy_buffer)(self.parent.id(), self.inner.get(), core::ptr::null())
    }
}

pub struct MapGuard<'d, 'b, T> {
    memory: NonZeroU64,
    ptr: NonNull<[T]>,
    buffer: &'b mut Buffer<'d, T>,
}

impl<T> Deref for MapGuard<'_, '_, T> {
    type Target = [T];

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { self.ptr.as_ref() }
    }
}

impl<T> DerefMut for MapGuard<'_, '_, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.ptr.as_mut() }
    }
}

impl<T> Drop for MapGuard<'_, '_, T> {
    #[inline]
    fn drop(&mut self) {
        let entry = Entry::get();
        (entry.unmap_memory)(self.buffer.parent.id(), self.memory.get());
        (entry.bind_buffer_memory)(self.buffer.parent.id(), self.buffer.id(), self.memory.get(), 0);
    }
}

unsafe impl<T: Send + Sync> Send for MapGuard<'_, '_, T> {}
unsafe impl<T: Sync> Sync for MapGuard<'_, '_, T> {}

bitflags::bitflags! {
    #[repr(transparent)]
    pub struct BufferFlags: vk::BufferCreateFlagBits {
        /// Buffer should support sparse backing
        const SPARSE_BINDING = vk::BUFFER_CREATE_SPARSE_BINDING_BIT;
        /// Buffer should support sparse backing with partial residency
        const SPARSE_RESIDENCY = vk::BUFFER_CREATE_SPARSE_RESIDENCY_BIT;
        /// Buffer should support constant data access to physical memory ranges mapped into multiple locations of sparse buffers
        const SPARSE_ALIASED = vk::BUFFER_CREATE_SPARSE_ALIASED_BIT;
        /// Buffer requires protected memory
        const PROTECTED = vk::BUFFER_CREATE_PROTECTED_BIT;
        const DEVICE_ADDRESS_CAPTURE_REPLAY = vk::BUFFER_CREATE_DEVICE_ADDRESS_CAPTURE_REPLAY_BIT;
        const DEVICE_ADDRESS_CAPTURE_REPLAY_EXT = vk::BUFFER_CREATE_DEVICE_ADDRESS_CAPTURE_REPLAY_BIT_EXT;
        const DEVICE_ADDRESS_CAPTURE_REPLAY_KHR = vk::BUFFER_CREATE_DEVICE_ADDRESS_CAPTURE_REPLAY_BIT_KHR;
    }
}

bitflags::bitflags! {
    #[repr(transparent)]
    pub struct UsageFlags: vk::BufferUsageFlagBits {
        /// Can be used as a source of transfer operations
        const TRANSFER_SRC = vk::BUFFER_USAGE_TRANSFER_SRC_BIT;
        /// Can be used as a destination of transfer operations
        const TRANSFER_DST = vk::BUFFER_USAGE_TRANSFER_DST_BIT;
        /// Can be used as TBO
        const UNIFORM_TEXEL_BUFFER = vk::BUFFER_USAGE_UNIFORM_TEXEL_BUFFER_BIT;
        /// Can be used as IBO
        const STORAGE_TEXEL_BUFFER = vk::BUFFER_USAGE_STORAGE_TEXEL_BUFFER_BIT;
        /// Can be used as UBO
        const UNIFORM_BUFFER = vk::BUFFER_USAGE_UNIFORM_BUFFER_BIT;
        /// Can be used as SSBO
        const STORAGE_BUFFER = vk::BUFFER_USAGE_STORAGE_BUFFER_BIT;
        /// Can be used as source of fixed-function index fetch (index buffer)
        const INDEX_BUFFER = vk::BUFFER_USAGE_INDEX_BUFFER_BIT;
        /// Can be used as source of fixed-function vertex fetch (VBO)
        const VERTEX_BUFFER = vk::BUFFER_USAGE_VERTEX_BUFFER_BIT;
        /// Can be the source of indirect parameters (e.g. indirect buffer, parameter buffer)
        const INDIRECT_BUFFER = vk::BUFFER_USAGE_INDIRECT_BUFFER_BIT;
        const SHADER_DEVICE_ADDRESS = vk::BUFFER_USAGE_SHADER_DEVICE_ADDRESS_BIT;
        const VIDEO_DECODE_SRC_KHR = vk::BUFFER_USAGE_VIDEO_DECODE_SRC_BIT_KHR;
        const VIDEO_DECODE_DST_KHR = vk::BUFFER_USAGE_VIDEO_DECODE_DST_BIT_KHR;
        const TRANSFORM_FEEDBACK_BUFFER_EXT = vk::BUFFER_USAGE_TRANSFORM_FEEDBACK_BUFFER_BIT_EXT;
        const TRANSFORM_FEEDBACK_COUNTER_BUFFER_EXT = vk::BUFFER_USAGE_TRANSFORM_FEEDBACK_COUNTER_BUFFER_BIT_EXT;
        /// Specifies the buffer can be used as predicate in conditional rendering
        const CONDITIONAL_RENDERING_EXT = vk::BUFFER_USAGE_CONDITIONAL_RENDERING_BIT_EXT;
        const ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR = vk::BUFFER_USAGE_ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_BIT_KHR;
        const ACCELERATION_STRUCTURE_STORAGE_KHR = vk::BUFFER_USAGE_ACCELERATION_STRUCTURE_STORAGE_BIT_KHR;
        const SHADER_BINDING_TABLE_KHR = vk::BUFFER_USAGE_SHADER_BINDING_TABLE_BIT_KHR;
        const RAY_TRACING_NV = vk::BUFFER_USAGE_RAY_TRACING_BIT_NV;
        const SHADER_DEVICE_ADDRESS_EXT = vk::BUFFER_USAGE_SHADER_DEVICE_ADDRESS_BIT_EXT;
        const SHADER_DEVICE_ADDRESS_KHR = vk::BUFFER_USAGE_SHADER_DEVICE_ADDRESS_BIT_KHR;
        const VIDEO_ENCODE_DST_KHR = vk::BUFFER_USAGE_VIDEO_ENCODE_DST_BIT_KHR;
        const VIDEO_ENCODE_SRC_KHR = vk::BUFFER_USAGE_VIDEO_ENCODE_SRC_BIT_KHR;
        const MICROMAP_BUILD_INPUT_READ_ONLY_EXT = vk::BUFFER_USAGE_MICROMAP_BUILD_INPUT_READ_ONLY_BIT_EXT;
        const MICROMAP_STORAGE_EXT = vk::BUFFER_USAGE_MICROMAP_STORAGE_BIT_EXT;
    }
}
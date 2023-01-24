use std::{marker::PhantomData, num::{NonZeroU64}, ptr::{addr_of, addr_of_mut, NonNull}, mem::{MaybeUninit, ManuallyDrop}, ops::{Deref, DerefMut, RangeBounds, Bound}, fmt::Debug};
use vk::{DeviceSize};
use crate::{Result, Entry, device::{Device}, alloc::{DeviceAllocator, MemoryPtr, MemoryFlags}, utils::u64_to_usize};

pub struct Buffer<T, A: DeviceAllocator> {
    buffer: NonZeroU64,
    memory: ManuallyDrop<MemoryPtr<A::Metadata>>,
    size: u64,
    alloc: A,
    _phtm: PhantomData<T>
}

impl<T, A: DeviceAllocator> Buffer<T, A> {
    pub const BYTES_PER_ELEMENT: vk::DeviceSize = core::mem::size_of::<T>() as vk::DeviceSize;

    pub fn new_uninit (capacity: DeviceSize, usage: UsageFlags, flags: BufferFlags, memory_flags: MemoryFlags, alloc: A) -> Result<Buffer<MaybeUninit<T>, A>> {
        let entry = Entry::get();
        let info = vk::BufferCreateInfo {
            sType: vk::STRUCTURE_TYPE_BUFFER_CREATE_INFO,
            pNext: core::ptr::null_mut(),
            flags: flags.bits(),
            size: capacity * Self::BYTES_PER_ELEMENT,
            usage: usage.bits(),
            sharingMode: vk::SHARING_MODE_EXCLUSIVE,
            queueFamilyIndexCount: 0,
            pQueueFamilyIndices: core::ptr::null(),
        };

        let mut inner = 0;
        tri! {
            (entry.create_buffer)(alloc.device().id(), addr_of!(info), core::ptr::null(), addr_of_mut!(inner))
        };

        if let Some(buffer) = NonZeroU64::new(inner) {
            let mut result = MaybeUninit::<vk::MemoryRequirements>::uninit();
            (Entry::get().get_buffer_memory_requirements)(
                alloc.device().id(),
                buffer.get(),
                result.as_mut_ptr()
            );

            let requirements = unsafe { result.assume_init() };
            let memory = alloc.allocate(requirements.size, requirements.alignment, memory_flags)?;
            tri! {
                (entry.bind_buffer_memory)(alloc.device().id(), buffer.get(), memory.id(), memory.range().start)
            };

            return Ok(Buffer { buffer, size: info.size, memory: ManuallyDrop::new(memory), alloc, _phtm: PhantomData })
        }

        return Err(vk::ERROR_INITIALIZATION_FAILED.into())
    }
    
    #[inline]
    pub fn from_sized_iter<I: IntoIterator<Item = T>> (iter: I, usage: UsageFlags, flags: BufferFlags, memory_flags: MemoryFlags, alloc: A) -> Result<Buffer<T, A>> where I::IntoIter: ExactSizeIterator {
        let iter = iter.into_iter();
        let mut this = Self::new_uninit(iter.len() as u64, usage, flags, memory_flags, alloc)?;
        
        let mut map = this.map_mut(..)?;
        for (map, value) in map.iter_mut().zip(iter) {
            let _ = map.write(value);
        }
        drop(map);
        
        return unsafe { Ok(this.assume_init()) }
    }

    #[inline]
    pub fn id (&self) -> u64 {
        return self.buffer.get()
    }

    #[inline]
    pub fn size (&self) -> u64 {
        return self.size
    }

    #[inline]
    pub fn len (&self) -> u64 {
        return self.size() / Self::BYTES_PER_ELEMENT
    }
    
    #[inline]
    pub fn device (&self) -> &Device {
        return self.alloc.device()
    }

    #[inline]
    pub fn alloc (&self) -> &A {
        return &self.alloc
    }

    #[inline]
    pub fn descriptor (&self) -> vk::DescriptorBufferInfo {
        return vk::DescriptorBufferInfo {
            buffer: self.buffer.get(),
            offset: 0,
            range: vk::WHOLE_SIZE,
        }
    }

    #[inline]
    pub fn map (&self, bounds: impl RangeBounds<usize>) -> Result<MapGuard<'_, T, A>> {
        unsafe {
            let ptr = self.map_ptr(bounds)?;
            return Ok(MapGuard { ptr, buffer: self })
        }
    }

    #[inline]
    pub fn map_mut (&mut self, bounds: impl RangeBounds<usize>) -> Result<MapMutGuard<'_, T, A>> {
        unsafe {
            let ptr = self.map_ptr(bounds)?;
            return Ok(MapMutGuard { ptr, buffer: self })
        }
    }

    #[inline]
    unsafe fn map_ptr (&self, bounds: impl RangeBounds<usize>) -> Result<NonNull<[T]>> {
        let start = match bounds.start_bound() {
            Bound::Excluded(x) => Bound::Excluded(*x * core::mem::size_of::<T>()),
            Bound::Included(x) => Bound::Included(*x * core::mem::size_of::<T>()),
            Bound::Unbounded => Bound::Unbounded
        };

        let end = match bounds.end_bound() {
            Bound::Excluded(x) => Bound::Excluded(*x * core::mem::size_of::<T>()),
            Bound::Included(x) => Bound::Included(*x * core::mem::size_of::<T>()),
            Bound::Unbounded => Bound::Excluded(u64_to_usize(self.size()))
        };
        
        let (ptr, size) = self.alloc.map(&self.memory, (start, end))?.to_raw_parts();        
        return Ok(NonNull::new_unchecked(core::ptr::from_raw_parts_mut::<[T]>(
            ptr.as_ptr(),
            size / core::mem::size_of::<T>()
        )));
    }
}

impl<T, A: DeviceAllocator> Buffer<T, A> {
    #[inline]
    pub unsafe fn transmute<U> (self) -> Buffer<U, A> {
        debug_assert_eq!(core::mem::size_of::<T>(), core::mem::size_of::<U>());
        let this = ManuallyDrop::new(self);
        return Buffer {
            buffer: this.buffer,
            size: this.size,
            memory: core::ptr::read(&this.memory),
            alloc: core::ptr::read(&this.alloc),
            _phtm: PhantomData
        }
    }

    #[inline]
    pub unsafe fn into_maybe_uninit (self) -> Buffer<MaybeUninit<T>, A> {
        self.transmute()
    }
}

impl<T, A: DeviceAllocator> Buffer<MaybeUninit<T>, A> {
    #[inline]
    pub unsafe fn assume_init (self) -> Buffer<T, A> {
        self.transmute()
    }
}

impl<'a, A: DeviceAllocator + Debug, T: Debug> Debug for Buffer<T, A> where A::Metadata: Debug {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Buffer")
            .field("buffer", &self.buffer)
            .field("memory", &self.memory)
            .field("alloc", &self.alloc)
            .field("_phtm", &self._phtm)
            .finish()
    }
}

impl<T, A: DeviceAllocator> Drop for Buffer<T, A> {
    #[inline]
    fn drop(&mut self) {
        unsafe { self.alloc.free(ManuallyDrop::take(&mut self.memory)) };
        (Entry::get().destroy_buffer)(self.device().id(), self.buffer.get(), core::ptr::null())
    }
}

pub struct MapGuard<'a, T, A: DeviceAllocator> {
    ptr: NonNull<[T]>,
    buffer: &'a Buffer<T, A>,
}

impl<T, A: DeviceAllocator> Deref for MapGuard<'_, T, A> {
    type Target = [T];

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { self.ptr.as_ref() }
    }
}

impl<T, A: DeviceAllocator> Drop for MapGuard<'_, T, A> {
    #[inline]
    fn drop(&mut self) {
        unsafe { self.buffer.alloc.unmap(&self.buffer.memory) }
    }
}

pub struct MapMutGuard<'a, T, A: DeviceAllocator> {
    ptr: NonNull<[T]>,
    buffer: &'a mut Buffer<T, A>,
}

impl<T, A: DeviceAllocator> MapMutGuard<'_, MaybeUninit<T>, A> {
    #[inline]
    pub fn init_from_slice (&mut self, slice: &[T]) where T: Copy {
        unsafe {
            self.copy_from_slice(&*(slice as *const [T] as *const [MaybeUninit<T>]));
        }
    }
}

impl<T, A: DeviceAllocator> Deref for MapMutGuard<'_, T, A> {
    type Target = [T];

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { self.ptr.as_ref() }
    }
}

impl<T, A: DeviceAllocator> DerefMut for MapMutGuard<'_, T, A> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.ptr.as_mut() }
    }
}

impl<T, A: DeviceAllocator> Drop for MapMutGuard<'_, T, A> {
    #[inline]
    fn drop(&mut self) {
        unsafe { self.buffer.alloc.unmap(&self.buffer.memory) }
    }
}

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
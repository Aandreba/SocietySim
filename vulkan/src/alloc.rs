use crate::{device::Device, Entry, Result};
use std::{
    fmt::Debug,
    marker::PhantomData,
    mem::{ManuallyDrop, MaybeUninit},
    num::NonZeroU64,
    ops::Range,
    ptr::{addr_of, addr_of_mut},
    sync::{Mutex, MutexGuard, TryLockError, Arc}, rc::Rc, panic::AssertUnwindSafe,
};
use vk::MemoryType;

pub trait MemoryMetadata {
    fn range(&self) -> Range<vk::DeviceSize>;
}

#[derive(Debug)]
pub struct MemoryPtr<M> {
    inner: NonZeroU64,
    _meta: M,
    _phtm: PhantomData<*mut ()>,
}

impl<M: MemoryMetadata> MemoryPtr<M> {
    #[inline]
    pub unsafe fn new(inner: NonZeroU64, meta: M) -> Self {
        return Self {
            inner,
            _meta: meta,
            _phtm: PhantomData,
        };
    }

    #[inline]
    pub fn id(&self) -> u64 {
        return self.inner.get();
    }

    #[inline]
    pub fn range(&self) -> Range<vk::DeviceSize> {
        return self._meta.range();
    }

    #[inline]
    pub fn size(&self) -> vk::DeviceSize {
        let range = self.range();
        return range.end - range.start;
    }
}

pub unsafe trait DeviceAllocator {
    type Metadata: MemoryMetadata;

    fn device(&self) -> &Device;
    fn allocate(
        &self,
        size: vk::DeviceSize,
        align: vk::DeviceSize,
        flags: MemoryFlags,
    ) -> Result<MemoryPtr<Self::Metadata>>;
    fn free(&self, ptr: MemoryPtr<Self::Metadata>);
}

unsafe impl<T: ?Sized + DeviceAllocator> DeviceAllocator for &T {
    type Metadata = T::Metadata;

    #[inline]
    fn device(&self) -> &Device {
        return T::device(*self)
    }

    #[inline]
    fn allocate(
        &self,
        size: vk::DeviceSize,
        align: vk::DeviceSize,
        flags: MemoryFlags,
    ) -> Result<MemoryPtr<Self::Metadata>> {
        return T::allocate(*self, size, align, flags)
    }

    #[inline]
    fn free(&self, ptr: MemoryPtr<Self::Metadata>) {
        return T::free(*self, ptr)
    }
}

unsafe impl<T: ?Sized + DeviceAllocator> DeviceAllocator for Box<T> {
    type Metadata = T::Metadata;

    #[inline]
    fn device(&self) -> &Device {
        return T::device(self)
    }

    #[inline]
    fn allocate(
        &self,
        size: vk::DeviceSize,
        align: vk::DeviceSize,
        flags: MemoryFlags,
    ) -> Result<MemoryPtr<Self::Metadata>> {
        return T::allocate(self, size, align, flags)
    }

    #[inline]
    fn free(&self, ptr: MemoryPtr<Self::Metadata>) {
        return T::free(self, ptr)
    }
}

unsafe impl<T: ?Sized + DeviceAllocator> DeviceAllocator for Rc<T> {
    type Metadata = T::Metadata;

    #[inline]
    fn device(&self) -> &Device {
        return T::device(self)
    }

    #[inline]
    fn allocate(
        &self,
        size: vk::DeviceSize,
        align: vk::DeviceSize,
        flags: MemoryFlags,
    ) -> Result<MemoryPtr<Self::Metadata>> {
        return T::allocate(self, size, align, flags)
    }

    #[inline]
    fn free(&self, ptr: MemoryPtr<Self::Metadata>) {
        return T::free(self, ptr)
    }
}

unsafe impl<T: ?Sized + DeviceAllocator> DeviceAllocator for Arc<T> {
    type Metadata = T::Metadata;

    #[inline]
    fn device(&self) -> &Device {
        return T::device(self)
    }

    #[inline]
    fn allocate(
        &self,
        size: vk::DeviceSize,
        align: vk::DeviceSize,
        flags: MemoryFlags,
    ) -> Result<MemoryPtr<Self::Metadata>> {
        return T::allocate(self, size, align, flags)
    }

    #[inline]
    fn free(&self, ptr: MemoryPtr<Self::Metadata>) {
        return T::free(self, ptr)
    }
}

unsafe impl<T: DeviceAllocator> DeviceAllocator for AssertUnwindSafe<T> {
    type Metadata = T::Metadata;

    #[inline]
    fn device(&self) -> &Device {
        return T::device(self)
    }

    #[inline]
    fn allocate(
        &self,
        size: vk::DeviceSize,
        align: vk::DeviceSize,
        flags: MemoryFlags,
    ) -> Result<MemoryPtr<Self::Metadata>> {
        return T::allocate(self, size, align, flags)
    }

    #[inline]
    fn free(&self, ptr: MemoryPtr<Self::Metadata>) {
        return T::free(self, ptr)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Raw<'a>(pub &'a Device);

unsafe impl DeviceAllocator for Raw<'_> {
    type Metadata = RawInfo;

    #[inline]
    fn device(&self) -> &Device {
        return self.0;
    }

    fn allocate<'a>(
        &self,
        size: vk::DeviceSize,
        _align: vk::DeviceSize,
        flags: MemoryFlags,
    ) -> Result<MemoryPtr<RawInfo>> {
        let entry = Entry::get();

        let mut props = MaybeUninit::uninit();
        (entry.get_physical_device_memory_properties)(self.0.physical().id(), props.as_mut_ptr());
        let props = unsafe { props.assume_init() };

        let mut info = None;
        for i in 0..props.memoryTypeCount {
            let MemoryType {
                propertyFlags,
                heapIndex,
            } = props.memoryTypes[i as usize];
            if MemoryFlags::from_bits_truncate(propertyFlags).contains(flags) {
                info = Some((props.memoryHeaps[heapIndex as usize].size, i));
                break;
            }
        }

        if let Some((_, mem_type_idx)) = info {
            let info = vk::MemoryAllocateInfo {
                sType: vk::STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO,
                pNext: core::ptr::null(),
                allocationSize: size,
                memoryTypeIndex: mem_type_idx,
            };

            let mut memory = 0;
            match (entry.allocate_memory)(
                self.0.id(),
                addr_of!(info),
                core::ptr::null(),
                addr_of_mut!(memory),
            ) {
                vk::SUCCESS => {}
                e => return Err(e.into()),
            }

            if let Some(inner) = NonZeroU64::new(memory) {
                return Ok(MemoryPtr {
                    inner,
                    _meta: RawInfo { size },
                    _phtm: PhantomData,
                });
            }
        }

        return Err(vk::ERROR_OUT_OF_DEVICE_MEMORY.into());
    }

    #[inline]
    fn free(&self, ptr: MemoryPtr<RawInfo>) {
        (Entry::get().free_memory)(self.device().id(), ptr.id(), core::ptr::null())
    }
}

#[derive(Debug)]
pub struct Page<'a> {
    inner: ManuallyDrop<MemoryPtr<RawInfo>>,
    flags: MemoryFlags,
    ranges: Mutex<Vec<Range<vk::DeviceSize>>>,
    alloc: Raw<'a>,
}

impl<'a> Page<'a> {
    #[inline]
    pub fn new(device: &'a Device, size: vk::DeviceSize, flags: MemoryFlags) -> Result<Self> {
        let raw = Raw(device);
        let inner = raw.allocate(size, 1, flags)?;
        return Ok(Self {
            inner: ManuallyDrop::new(inner),
            flags,
            alloc: raw,
            ranges: Mutex::new(vec![0..size]),
        });
    }

    #[inline]
    fn try_allocate(
        &self,
        size: vk::DeviceSize,
        align: vk::DeviceSize,
        _flags: MemoryFlags,
    ) -> Result<Option<MemoryPtr<PageInfo>>> {
        debug_assert_eq!(_flags, self.flags);

        return match self.ranges.try_lock() {
            Ok(ranges) => Self::inner_allocate(self, ranges, size, align).map(Some),
            Err(TryLockError::Poisoned(e)) => {
                Self::inner_allocate(self, e.into_inner(), size, align).map(Some)
            }
            Err(_) => Ok(None),
        };
    }

    fn inner_allocate(
        &self,
        mut ranges: MutexGuard<'_, Vec<Range<vk::DeviceSize>>>,
        size: vk::DeviceSize,
        align: vk::DeviceSize,
    ) -> Result<MemoryPtr<PageInfo>> {
        let mut result = None;
        for i in 0..ranges.len() {
            let range = unsafe { ranges.get_unchecked(i) };
            let padding = range.start % align;
            let chunk_size = range.end - range.start;
            if chunk_size > padding + size {
                result = Some((i, padding))
            }
        }

        if let Some((i, padding)) = result {
            let mut range = unsafe { ranges.get_unchecked_mut(i) };
            
            if padding == 0 {
                let start = range.start;
                let end = range.start + size;

                range.start = end;
                return unsafe {
                    Ok(MemoryPtr::new(
                        self.inner.inner,
                        PageInfo {
                            range: start..end,
                        },
                    ))
                };
            } else {
                let start = range.start + padding;
                let end = start + size;

                let prev_end = core::mem::replace(&mut range.end, padding);
                ranges.push(end..prev_end);

                return unsafe {
                    Ok(MemoryPtr::new(
                        self.inner.inner,
                        PageInfo {
                            range: start..end,
                        },
                    ))
                };
            }

        } else {
            return Err(vk::ERROR_OUT_OF_DEVICE_MEMORY.into());
        }
    }
}

unsafe impl DeviceAllocator for Page<'_> {
    type Metadata = PageInfo;

    #[inline]
    fn device(&self) -> &Device {
        return self.alloc.device();
    }

    #[inline]
    fn allocate(
        &self,
        size: vk::DeviceSize,
        align: vk::DeviceSize,
        _flags: MemoryFlags,
    ) -> Result<MemoryPtr<PageInfo>> {
        debug_assert_eq!(_flags, self.flags);
        let ranges = match self.ranges.lock() {
            Ok(x) => x,
            Err(e) => e.into_inner(),
        };

        return Self::inner_allocate(self, ranges, size, align);
    }

    #[inline]
    fn free(&self, ptr: MemoryPtr<PageInfo>) {
        let mut ranges = match self.ranges.lock() {
            Ok(x) => x,
            Err(e) => e.into_inner(),
        };
        ranges.push(ptr._meta.range)
    }
}

impl Drop for Page<'_> {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            self.alloc.free(ManuallyDrop::take(&mut self.inner))
        }
    }
}

unsafe impl Send for Page<'_> {}
unsafe impl Sync for Page<'_> {}

/// Metadata for [`Raw`]-allocated memory
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RawInfo {
    size: vk::DeviceSize,
}

impl MemoryMetadata for RawInfo {
    #[inline]
    fn range(&self) -> Range<vk::DeviceSize> {
        0..self.size
    }
}

/// Metadata for [`Page`]-allocated memory
pub struct PageInfo {
    range: Range<vk::DeviceSize>,
}

impl MemoryMetadata for PageInfo {
    #[inline]
    fn range(&self) -> Range<vk::DeviceSize> {
        self.range.clone()
    }
}

bitflags::bitflags! {
    #[repr(transparent)]
    pub struct MemoryFlags: vk::MemoryPropertyFlagBits {
        /// If otherwise stated, then allocate memory on device
        const DEVICE_LOCAL = vk::MEMORY_PROPERTY_DEVICE_LOCAL_BIT;
        /// Memory is mappable by host
        const HOST_VISIBLE = vk::MEMORY_PROPERTY_HOST_VISIBLE_BIT;
        /// Memory will have i/o coherency. If not set, application may need to use `vkFlushMappedMemoryRanges` and `vkInvalidateMappedMemoryRanges` to flush/invalidate host cache
        const HOST_COHERENT = vk::MEMORY_PROPERTY_HOST_COHERENT_BIT;
        /// Memory will be cached by the host
        const HOST_CACHED = vk::MEMORY_PROPERTY_HOST_CACHED_BIT;
        /// Memory may be allocated by the driver when it is required
        const LAZILY_ALLOCATED = vk::MEMORY_PROPERTY_LAZILY_ALLOCATED_BIT;
        /// Memory is protected
        const PROTECTED = vk::MEMORY_PROPERTY_PROTECTED_BIT;
        const DEVICE_COHERENT_AMD = vk::MEMORY_PROPERTY_DEVICE_COHERENT_BIT_AMD;
        const DEVICE_UNCACHED_AMD = vk::MEMORY_PROPERTY_DEVICE_UNCACHED_BIT_AMD;
        const RDMA_CAPABLE_NV = vk::MEMORY_PROPERTY_RDMA_CAPABLE_BIT_NV;

        // https://gpuopen.com/learn/vulkan-device-memory/
        const MAPABLE = vk::MEMORY_PROPERTY_DEVICE_LOCAL_BIT | vk::MEMORY_PROPERTY_HOST_VISIBLE_BIT | vk::MEMORY_PROPERTY_HOST_COHERENT_BIT;
    }
}

impl Default for MemoryFlags {
    #[inline]
    fn default() -> Self {
        Self::DEVICE_LOCAL
    }
}

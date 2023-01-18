use crate::{device::{Device, DeviceRef}, Entry, Result, utils::u64_to_usize};
use std::{
    fmt::Debug,
    marker::PhantomData,
    mem::{ManuallyDrop, MaybeUninit},
    num::NonZeroU64,
    ops::{Range, RangeBounds, Bound},
    ptr::{addr_of, addr_of_mut, NonNull},
    sync::{Mutex, MutexGuard, TryLockError, Arc, atomic::{AtomicPtr, Ordering}}, rc::Rc, ffi::c_void,
};
use vk::MemoryType;

const UNINIT: *mut c_void = core::ptr::null_mut();
const INITIALIZING: *mut c_void = NonNull::dangling().as_ptr();

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
    
    unsafe fn free(&self, ptr: MemoryPtr<Self::Metadata>);

    /// # Safety
    /// It is up to the caller to ensure that Rust's [borrowing rules](https://doc.rust-lang.org/stable/book/ch04-02-references-and-borrowing.html) are followed for the maps.
    unsafe fn map (&self, mem: &MemoryPtr<Self::Metadata>, bounds: impl RangeBounds<usize>) -> Result<NonNull<[u8]>>;
    /// # Safety
    /// It is up to the caller to ensure that Rust's [borrowing rules](https://doc.rust-lang.org/stable/book/ch04-02-references-and-borrowing.html) are followed for the maps.
    unsafe fn unmap (&self, mem: &MemoryPtr<Self::Metadata>);
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
    unsafe fn free(&self, ptr: MemoryPtr<Self::Metadata>) {
        return T::free(*self, ptr)
    }

    #[inline]
    unsafe fn map (&self, mem: &MemoryPtr<Self::Metadata>, bounds: impl RangeBounds<usize>) -> Result<NonNull<[u8]>> {
        T::map(*self, mem, bounds)
    }

    #[inline]
    unsafe fn unmap (&self, mem: &MemoryPtr<Self::Metadata>) {
        T::unmap(*self, mem)
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
    unsafe fn free(&self, ptr: MemoryPtr<Self::Metadata>) {
        return T::free(self, ptr)
    }

    #[inline]
    unsafe fn map (&self, mem: &MemoryPtr<Self::Metadata>, bounds: impl RangeBounds<usize>) -> Result<NonNull<[u8]>> {
        T::map(self, mem, bounds)
    }

    #[inline]
    unsafe fn unmap (&self, mem: &MemoryPtr<Self::Metadata>) {
        T::unmap(self, mem)
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
    unsafe fn free(&self, ptr: MemoryPtr<Self::Metadata>) {
        return T::free(self, ptr)
    }

    #[inline]
    unsafe fn map (&self, mem: &MemoryPtr<Self::Metadata>, bounds: impl RangeBounds<usize>) -> Result<NonNull<[u8]>> {
        T::map(self, mem, bounds)
    }

    #[inline]
    unsafe fn unmap (&self, mem: &MemoryPtr<Self::Metadata>) {
        T::unmap(self, mem)
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
    unsafe fn free(&self, ptr: MemoryPtr<Self::Metadata>) {
        return T::free(self, ptr)
    }

    #[inline]
    unsafe fn map (&self, mem: &MemoryPtr<Self::Metadata>, bounds: impl RangeBounds<usize>) -> Result<NonNull<[u8]>> {
        T::map(self, mem, bounds)
    }

    #[inline]
    unsafe fn unmap (&self, mem: &MemoryPtr<Self::Metadata>) {
        T::unmap(self, mem)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct RawInner<D> (pub D);

unsafe impl<D: DeviceRef> DeviceAllocator for RawInner<D> {
    type Metadata = RawInfo;

    #[inline]
    fn device(&self) -> &Device {
        return self.0.deref();
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
    unsafe fn free(&self, ptr: MemoryPtr<RawInfo>) {
        (Entry::get().free_memory)(self.device().id(), ptr.id(), core::ptr::null())
    }

    unsafe fn map (&self, mem: &MemoryPtr<Self::Metadata>, bounds: impl RangeBounds<usize>) -> Result<NonNull<[u8]>> {
        let entry = Entry::get();

        let start = match bounds.start_bound() {
            Bound::Excluded(x) => *x + 1,
            Bound::Included(x) => *x,
            Bound::Unbounded => 0
        };

        let end = match bounds.end_bound() {
            Bound::Excluded(x) => *x,
            Bound::Included(x) => *x + 1,
            #[cfg(debug_assertions)]
            Bound::Unbounded => usize::try_from(mem._meta.size).unwrap(),
            #[cfg(not(debug_assertions))]
            Bound::Unbounded => mem._meta.size as usize
        };
        
        let len = end - start;
        let mut ptr: *mut c_void = core::ptr::null_mut();
        (entry.map_memory)(self.device().id(), mem.id(), start as u64, len as u64, 0, addr_of_mut!(ptr));
        
        if let Some(ptr) = NonNull::new(ptr) {
            let ptr = ptr.as_ptr().byte_add(start);
            debug_assert!(!ptr.is_null());

            return Ok(NonNull::new_unchecked(core::ptr::from_raw_parts_mut::<[u8]>(
                ptr.cast(),
                len
            )))
        }

        return Err(vk::ERROR_MEMORY_MAP_FAILED.into())
    }

    #[inline]
    unsafe fn unmap (&self, mem: &MemoryPtr<Self::Metadata>) {
        (Entry::get().unmap_memory)(self.device().id(), mem.id())
    }
}

#[derive(Debug)]
pub struct Page<D: DeviceRef> {
    inner: ManuallyDrop<MemoryPtr<RawInfo>>,
    flags: MemoryFlags,
    ranges: Mutex<Vec<Range<vk::DeviceSize>>>,
    mapped_ptr: AtomicPtr<c_void>,
    alloc: RawInner<D>,
}

impl<D: DeviceRef> Page<D> {
    #[inline]
    pub fn new(device: D, size: vk::DeviceSize, flags: MemoryFlags) -> Result<Self> {
        let raw = RawInner(device);
        let inner = raw.allocate(size, 1, flags)?;
        return Ok(Self {
            inner: ManuallyDrop::new(inner),
            ranges: Mutex::new(vec![0..size]),
            mapped_ptr: AtomicPtr::new(UNINIT),
            alloc: raw,
            flags,
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

unsafe impl<D: DeviceRef> DeviceAllocator for Page<D> {
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
    unsafe fn free(&self, ptr: MemoryPtr<PageInfo>) {
        let mut ranges = match self.ranges.lock() {
            Ok(x) => x,
            Err(e) => e.into_inner(),
        };
        
        let mut ptr_range = ptr._meta.range;
        let mut i = 0;

        while i < ranges.len() {
            let range = unsafe { ranges.get_unchecked_mut(i) };
            
            if range.start == ptr_range.end {
                range.start = ptr_range.start;
                ptr_range = ranges.swap_remove(i);
                i = 0;
                continue;
            }
            
            if range.end == ptr_range.start {
                range.end = ptr_range.end;
                ptr_range = ranges.swap_remove(i);
                i = 0;
                continue;
            }

            i += 1;
        }

        ranges.push(ptr_range)
    }

    unsafe fn map (&self, mem: &MemoryPtr<Self::Metadata>, bounds: impl RangeBounds<usize>) -> Result<NonNull<[u8]>> {        
        // Obtained general mapped pointer
        let ptr = loop {
            match self.mapped_ptr.compare_exchange(UNINIT, INITIALIZING, Ordering::AcqRel, Ordering::Acquire) {
                // Map the full region
                Ok(_) => {
                    let ptr = match self.alloc.map(&self.inner, ..) {
                        Ok(x) => x.cast::<c_void>(),
                        Err(e) => {
                            self.mapped_ptr.store(UNINIT, Ordering::Release);
                            return Err(e)
                        }
                    };
                    
                    self.mapped_ptr.store(ptr.as_ptr(), Ordering::Release);
                    break ptr
                },

                // Wait until mapping is done
                Err(INITIALIZING) => core::hint::spin_loop(),
                // Get initialized mapping
                Err(other) => break unsafe { NonNull::new_unchecked(other) }
            }
        };

        // Calculate start & end points
        let offset = u64_to_usize(mem._meta.range.start);
        let start = offset + match bounds.start_bound() {
            Bound::Excluded(x) => *x + 1,
            Bound::Included(x) => *x,
            Bound::Unbounded => 0
        };
        let end = match bounds.end_bound() {
            Bound::Excluded(x) => offset + *x,
            Bound::Included(x) => offset + *x + 1,
            Bound::Unbounded => u64_to_usize(mem._meta.range.end),
        };

        // Check that mapped bounds are contained inside buffer
        if (start as u64) > mem._meta.range.end || (end as u64) > mem._meta.range.end {
            #[cfg(debug_assertions)]
            eprintln!("Bounds overflow");
            return Err(vk::ERROR_MEMORY_MAP_FAILED.into())
        }

        let ptr = ptr.as_ptr().byte_add(start);
        debug_assert!(!ptr.is_null());

        return Ok(NonNull::new_unchecked(core::ptr::from_raw_parts_mut(
            ptr.cast(),
            end - start
        )));
    }

    #[inline]
    unsafe fn unmap (&self, _mem: &MemoryPtr<Self::Metadata>) {
        // noop
    }
}

impl<D: DeviceRef> Drop for Page<D> {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            match *self.mapped_ptr.get_mut() {
                UNINIT | INITIALIZING => {},
                _ => self.alloc.unmap(&self.inner) 
            }
            self.alloc.free(ManuallyDrop::take(&mut self.inner))
        }
    }
}

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

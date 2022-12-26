use std::{num::NonZeroU64, marker::PhantomData, mem::MaybeUninit, ptr::{addr_of, addr_of_mut}, future::{ready, Ready}};
use futures::{Future};
use vk::MemoryType;
use crate::{Entry, Result, device::Device};

#[derive(Debug)]
pub struct MemoryPtr<'a, A: ?Sized + DeviceAllocator> {
    inner: NonZeroU64,
    device: &'a Device,
    _meta: A::Metadata,
    _phtm: PhantomData<*mut ()>
}

impl<'a, A: ?Sized + DeviceAllocator> MemoryPtr<'a, A> {
    #[inline]
    pub unsafe fn new (inner: NonZeroU64, device: &'a Device, meta: A::Metadata) -> Self {
        return Self {
            inner,
            device,
            _meta: meta,
            _phtm: PhantomData,
        }
    }
}

impl<A: ?Sized + DeviceAllocator> MemoryPtr<'_, A> {
    #[inline]
    pub fn id (&self) -> u64 {
        return self.inner.get()
    }

    #[inline]
    pub fn device (&self) -> &Device {
        return self.device
    }
}

pub unsafe trait DeviceAllocator {
    type Metadata;
    type Allocate<'a, 'b>: 'a + Future<Output = Result<MemoryPtr<'a, Self>>> where Self: 'b;

    fn allocate<'a, 'b> (&'b self, device: &'a Device, size: vk::DeviceSize, align: vk::DeviceSize, flags: MemoryFlags) -> Self::Allocate<'a, 'b>;
    fn free (&self, ptr: MemoryPtr<Self>);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Raw;

unsafe impl DeviceAllocator for Raw {
    type Metadata = ();
    type Allocate<'a, 'b> = Ready<Result<MemoryPtr<'a, Self>>> where Self: 'b;

    fn allocate<'a, 'b> (&'b self, device: &'a Device, size: vk::DeviceSize, _align: vk::DeviceSize, flags: MemoryFlags) -> Self::Allocate<'a, 'b> {
        let entry = Entry::get();

        let mut props = MaybeUninit::uninit();
        (entry.get_physical_device_memory_properties)(device.physical().id(), props.as_mut_ptr());
        let props = unsafe { props.assume_init() };

        let mut info = None;
        for i in 0..props.memoryTypeCount {
            let MemoryType { propertyFlags, heapIndex } = props.memoryTypes[i as usize];
            if MemoryFlags::from_bits_truncate(propertyFlags).contains(flags) {
                info = Some((props.memoryHeaps[heapIndex as usize].size, i));
                break
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
            match (entry.allocate_memory)(device.id(), addr_of!(info), core::ptr::null(), addr_of_mut!(memory)) {
                vk::SUCCESS => {},
                e => return ready(Err(e.into()))
            }

            if let Some(inner) = NonZeroU64::new(memory) {
                return ready(Ok(MemoryPtr { inner, device, _meta: (), _phtm: PhantomData }))
            }
        }
        
        return ready(Err(vk::ERROR_OUT_OF_DEVICE_MEMORY.into()))
    }

    #[inline]
    fn free (&self, ptr: MemoryPtr<Self>) {
        (Entry::get().free_memory)(ptr.device.id(), ptr.id(), core::ptr::null())
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
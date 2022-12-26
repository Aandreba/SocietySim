use std::{alloc::Allocator, ptr::NonNull};
use crate::{Entry, device::Device};

pub struct Vulkan<'a> {
    device: &'a Device,
    cbs: Option<vk::AllocationCallbacks>
}

impl<'a> Vulkan<'a> {
    
}

unsafe impl Allocator for Vulkan<'_> {
    #[inline]
    fn allocate(&self, layout: std::alloc::Layout) -> Result<std::ptr::NonNull<[u8]>, std::alloc::AllocError> {
        let cbs = match self.cbs {
            Some(ref x) => x as *const vk::AllocationCallbacks,
            None => core::ptr::null()
        };

        let info = vk::MemoryAllocateInfo {
            sType: todo!(),
            pNext: core::ptr::null_mut(),
            allocationSize: todo!(),
            memoryTypeIndex: todo!(),
        };

        let alloc = (Entry::get().allocate_memory)();
    }

    unsafe fn deallocate(&self, ptr: std::ptr::NonNull<u8>, layout: std::alloc::Layout) {
        todo!()
    }

    unsafe fn grow(
            &self,
            ptr: NonNull<u8>,
            old_layout: std::alloc::Layout,
            new_layout: std::alloc::Layout,
        ) -> Result<NonNull<[u8]>, std::alloc::AllocError> {
        todo!()
    }

    unsafe fn shrink(
            &self,
            ptr: NonNull<u8>,
            old_layout: std::alloc::Layout,
            new_layout: std::alloc::Layout,
        ) -> Result<NonNull<[u8]>, std::alloc::AllocError> {
        todo!()
    }
}
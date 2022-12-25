use std::{collections::HashMap, ffi::c_void, alloc::{Layout, Allocator}};

struct VulkanAllocator<A> {
    map: HashMap<*mut c_void, Layout>,
    alloc: A
}

impl<A: Allocator> VulkanAllocator<A> {
    #[inline]
    pub const fn new (alloc: A) -> Self {
        return Self {
            map: HashMap::new(),
            alloc
        }
    }
}

pub fn vulkan_allocator<A: Allocator> (alloc: &'static VulkanAllocator<A>) -> crate::vk::AllocationCallbacks {
    use std::{ffi::c_void, alloc::Layout};

    extern "system" fn alloc_fn<A: std::alloc::Allocator> (data: *mut c_void, size: usize, align: usize, _: vk::SystemAllocationScope) -> *mut c_void {
        unsafe {
            let this = &*(data as *const VulkanAllocator<A>);
            #[cfg(not(debug_assertions))]
            let layout = Layout::from_size_align_unchecked(size, align);
            #[cfg(debug_assertions)]
            let layout = match Layout::from_size_align(size, align) {
                Ok(x) => x,
                Err(e) => {
                    eprintln!("{e}");
                    return core::ptr::null_mut()
                }
            };

            return match A::allocate(&this.alloc, layout) {
                Ok(x) => x.as_ptr().cast(),
                Err(e) => {
                    eprintln!("{e}");
                    core::ptr::null_mut()
                }
            };
        }
    }

    extern "system" fn free_fn<A: std::alloc::Allocator> (data: *mut c_void, ptr: *mut c_void) {
        unsafe {
            let this = &*(data as *const VulkanAllocator<A>);
            if let Some(layout) = this.map.
        }

        todo!()
    }

    extern "system" fn realloc_fn<A: std::alloc::Allocator> (data: *mut c_void, ptr: *mut c_void, size: usize, align: usize, _: vk::SystemAllocationScope) -> *mut c_void {
        unsafe {
            let this = &*(data as *const A);
            #[cfg(not(debug_assertions))]
            let layout = Layout::from_size_align_unchecked(size, align);
            #[cfg(debug_assertions)]
            let layout = match Layout::from_size_align(size, align) {
                Ok(x) => x,
                Err(e) => {
                    eprintln!("{e}");
                    return core::ptr::null_mut()
                }
            };

            A::grow(&self, ptr, old_layout, new_layout)
        }
    }

    return crate::vk::AllocationCallbacks {
        pUserData: alloc as *const VulkanAllocator<A> as *mut c_void,
        pfnAllocation: alloc_fn::<A>,
        pfnReallocation: todo!(),
        pfnFree: free_fn::<A>,
        pfnInternalAllocation: todo!(),
        pfnInternalFree: todo!()
    }
}
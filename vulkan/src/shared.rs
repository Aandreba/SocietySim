use std::{ptr::{NonNull, addr_of, addr_of_mut}, mem::MaybeUninit, ffi::c_void, num::NonZeroU64};
use proc::cstr;

use crate::{Entry, device::DeviceRef, Result, alloc::{RawInner}};

#[derive(Debug, Clone, Copy)]
pub struct SharedPtr<T: ?Sized, D> {
    ptr: NonNull<T>,
    alloc: RawInner<D>
}

impl<T: ?Sized, D: DeviceRef> SharedPtr<T, D> {
    pub fn from_ptr (device: D, ptr: NonNull<T>) -> Result<Self> {
        let mut props = MaybeUninit::uninit();
        tri! {
            (Entry::get().get_memory_host_pointer_properties_e_x_t)(
                device.id(),
                vk::EXTERNAL_MEMORY_HANDLE_TYPE_HOST_ALLOCATION_BIT_EXT,
                ptr.as_ptr() as *const c_void,
                props.as_mut_ptr()
            )
        }
        let props = unsafe { props.assume_init() };
        println!("{:#?}, {:#?}, {:#?}", props.sType, props.pNext, props.memoryTypeBits);

        let mut phy_props = MaybeUninit::uninit();
        (Entry::get().get_physical_device_memory_properties)(device.physical().id(), phy_props.as_mut_ptr());
        let phy_props = unsafe { phy_props.assume_init() };

        let mut info = None;
        for i in 0..phy_props.memoryTypeCount {
            let vk::MemoryType {
                propertyFlags,
                heapIndex,
            } = phy_props.memoryTypes[i as usize];
            if propertyFlags & props.memoryTypeBits == props.memoryTypeBits {
                info = Some((phy_props.memoryHeaps[heapIndex as usize].size, i));
                break;
            }
        }

        if let Some((_, mem_type_idx)) = info {
            let import = vk::ImportMemoryHostPointerInfoEXT {
                sType: vk::STRUCTURE_TYPE_IMPORT_MEMORY_HOST_POINTER_INFO_EXT,
                pNext: core::ptr::null(),
                handleType: vk::EXTERNAL_MEMORY_HANDLE_TYPE_HOST_ALLOCATION_BIT_EXT,
                pHostPointer: ptr.as_ptr().cast(),
            };
    
            let info = vk::MemoryAllocateInfo {
                sType: vk::STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO,
                pNext: addr_of!(import).cast(),
                allocationSize: unsafe { core::mem::size_of_val_raw(ptr.as_ptr()) as u64 },
                memoryTypeIndex: mem_type_idx,
            };

            let mut memory = 0;
            match (Entry::get().allocate_memory)(
                device.id(),
                addr_of!(info),
                core::ptr::null(),
                addr_of_mut!(memory),
            ) {
                vk::SUCCESS => {}
                e => return Err(e.into()),
            }

            if let Some(inner) = NonZeroU64::new(memory) {
                todo!()
                // return Ok(MemoryPtr {
                //    inner,
                //    _meta: RawInfo { size },
                //    _phtm: PhantomData,
                //});
            }
        }

        return Err(vk::ERROR_OUT_OF_DEVICE_MEMORY.into());
    }
}

#[test]
fn shared_ptr () -> anyhow::Result<()> {
    let _ = unsafe {
        Entry::builder(1, 1, 0)
            .extensions([cstr!("VK_EXT_external_memory_host")])
            .build()?
    };

    let phy = crate::physical_dev::PhysicalDevice::first()?;
    let (dev, _) = crate::device::Device::builder(phy).queues(&[1f32]).build().build()?;

    let mut ptr = Box::new(3);
    let shared = SharedPtr::from_ptr(&dev, NonNull::new(&mut ptr as &mut i32).unwrap())?;
    println!("{shared:#?}");

    return Ok(())
}
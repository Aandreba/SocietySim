#![cfg_attr(docsrs, feature(doc_cfg))]
#![feature(ptr_metadata)]

use std::mem::MaybeUninit;

use vulkan::{Entry, device::{Device}, buffer::{UsageFlags, BufferFlags}, physical_dev::PhysicalDevice, alloc::{MemoryFlags, Raw}};

#[macro_export]
macro_rules! flat_mod {
    ($($i:ident),+) => {
        $(
            mod $i;
            pub use $i::*;
        )+
    };
}

#[allow(unused)]
#[macro_export]
macro_rules! cstr {
    ($l:literal) => {unsafe {
        core::ffi::CStr::from_bytes_with_nul_unchecked(
            concat!($l, "\0").as_bytes()
        )
    }};
}

#[tokio::main]
async fn main () -> anyhow::Result<()> {
    let _ = unsafe { Entry::builder(1, 0, 0).build_in("/opt/homebrew/Cellar/molten-vk/1.2.1/lib/libMoltenVK.dylib") }?;
    
    let phy = PhysicalDevice::first()?;
    let (dev, _) = Device::builder(phy)
        .queues(&[1f32]).build()
        .build()?;

    let mut buffer = dev.create_buffer_uninit::<f32, _>(
        5,
        UsageFlags::STORAGE_BUFFER,
        BufferFlags::empty(),
        MemoryFlags::MAPABLE,
        Raw
    ).await?;

    let mut map = buffer.map(..)?;
    for (i, v) in map.iter_mut().enumerate() {
        v.write(i as f32);
    }

    drop(map);
    let mut buffer = unsafe { buffer.assume_init() };
    let map = buffer.map(..)?;
    println!("{:#?}", &map as &[f32]);


    Ok(())
}
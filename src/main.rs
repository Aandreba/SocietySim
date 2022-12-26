#![cfg_attr(docsrs, feature(doc_cfg))]
#![feature(ptr_metadata)]

use vulkan::{Entry, device::{Device}, buffer::{UsageFlags, BufferFlags}, physical_dev::PhysicalDevice, alloc::{MemoryFlags, Raw}, shader::Module};

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

    let mut lhs = dev.create_buffer_uninit::<u32, _>(
        5,
        UsageFlags::STORAGE_BUFFER,
        BufferFlags::empty(),
        MemoryFlags::MAPABLE,
        Raw
    ).await?;

    let mut rhs = dev.create_buffer_uninit::<u32, _>(
        5,
        UsageFlags::STORAGE_BUFFER,
        BufferFlags::empty(),
        MemoryFlags::MAPABLE,
        Raw
    ).await?;

    lhs.map(..)?.init_from_slice(&[1, 2, 3, 4, 5]);
    rhs.map(..)?.init_from_slice(&[6, 7, 8, 9, 10]);

    let lhs = unsafe { lhs.assume_init() };
    let rhs = unsafe { rhs.assume_init() };
    
    let shader = Module::from_bytes(&dev, include_bytes!("../target/main.spv"))?;
    println!("{shader:#?}");

    Ok(())
}
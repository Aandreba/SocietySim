#![cfg_attr(docsrs, feature(doc_cfg))]
#![feature(ptr_metadata)]

use vulkan::{Entry, device::{Device}, buffer::{UsageFlags, BufferFlags}, physical_dev::PhysicalDevice, alloc::{MemoryFlags, Raw}, shader::{Module, Shader, BindingType, StageFlags}, pipeline::Pipeline};

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
    
    let shader = Shader::builder(&dev)
        .binding(BindingType::StorageBuffer, 1, StageFlags::COMPUTE)
        .binding(BindingType::StorageBuffer, 1, StageFlags::COMPUTE)
    .build(include_bytes!("../target/main.spv"))?;

    let pipeline = Pipeline::compute(&shader, cstr!("main")).build_compute()?;

    println!("{pipeline:#?}");

    Ok(())
}
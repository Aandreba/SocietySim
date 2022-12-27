#![cfg_attr(docsrs, feature(doc_cfg))]
#![feature(ptr_metadata)]

use vulkan::{Entry, device::{Device}, physical_dev::PhysicalDevice, shader::{Shader, ShaderStage}, pipeline::Pipeline, descriptor::{DescriptorType, DescriptorPool, DescriptorSet}, utils::read_spv_tokio};

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

    let mut file = tokio::fs::File::open("target/main.spv").await.unwrap();
    let words = read_spv_tokio(&mut file).await.unwrap();

    let shader = Shader::builder(&dev, ShaderStage::COMPUTE)
        .binding(DescriptorType::StorageBuffer, 1)
        .binding(DescriptorType::StorageBuffer, 1)
    .build(&words)?;

    
    let pipeline = Pipeline::compute(&shader).build()?;
    
    let pool = DescriptorPool::builder(&dev, 1)
        .pool_size(DescriptorType::StorageBuffer, 2)
        .build()?;

    let set = DescriptorSet::new(&pool);
    
    println!("{pipeline:#?}");
    println!("{pool:#?}");

    Ok(())
}
#![cfg_attr(docsrs, feature(doc_cfg))]
#![feature(ptr_metadata)]

use vulkan::{Entry, device::{Device}, physical_dev::PhysicalDevice, pipeline::{ComputeBuilder}, descriptor::{DescriptorType, DescriptorPool}, utils::{read_spv}, buffer::{Buffer, UsageFlags, BufferFlags}, alloc::{MemoryFlags, Raw}, pool::{CommandPool, CommandPoolFlags, CommandBufferLevel}};

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

fn main () -> anyhow::Result<()> {
    //let _ = unsafe { Entry::builder(1, 0, 0).build_in("/opt/homebrew/Cellar/molten-vk/1.2.1/lib/libMoltenVK.dylib") }?;
    let _ = unsafe { Entry::builder(1, 0, 0).build() }?;
    
    let phy = PhysicalDevice::first()?;
    let family = phy.families().next().unwrap();
    let dev = Device::builder(phy)
        .queues(&[1f32]).build()
        .build()?;

    let mut input = Buffer::new_uninit(&dev, 5, UsageFlags::STORAGE_BUFFER, BufferFlags::empty(), MemoryFlags::MAPABLE, Raw)?;
    let output = Buffer::<f32, _>::new_uninit(&dev, 5, UsageFlags::STORAGE_BUFFER, BufferFlags::empty(), MemoryFlags::MAPABLE, Raw)?;

    input.map(..)?.init_from_slice(&[1f32, 2f32, 3f32, 4f32, 5f32]);
    let input = unsafe { input.assume_init() };

    let mut file = std::fs::File::open("target/main.spv")?;
    let words = read_spv(&mut file)?;
    
    let mut pipeline = ComputeBuilder::new(&dev)
        .binding(DescriptorType::StorageBuffer, 1)
        .binding(DescriptorType::StorageBuffer, 1)
        .build(&words)?;
    
    let set = pipeline.sets().first().unwrap();
    let input_desc = set.write_descriptor(&input, 0);
    let output_desc = set.write_descriptor(&output, 0);
    pipeline.sets_mut().update(&[input_desc, output_desc]);

    let pool = DescriptorPool::builder(&dev, 1)
        .pool_size(DescriptorType::StorageBuffer, 2)
        .build()?;
    
    let cmd = CommandPool::new(&dev, family, CommandPoolFlags::empty())?;
    let mut cmd_buff = cmd.allocate_buffers(1, CommandBufferLevel::Primary)?;
    let cmd_buff = &mut cmd_buff[0];

    Ok(())
}
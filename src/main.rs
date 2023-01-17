#![cfg_attr(docsrs, feature(doc_cfg))]
#![feature(ptr_metadata)]

use vulkan::{Entry, device::{Device}, physical_dev::PhysicalDevice, pipeline::{ComputeBuilder}, descriptor::{DescriptorType}, buffer::{Buffer, UsageFlags, BufferFlags}, alloc::{MemoryFlags, Page}, pool::{CommandPool, CommandPoolFlags, CommandBufferLevel, CommandBufferUsage, PipelineBindPoint}, queue::{Fence, FenceFlags}, include_spv};

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

    let alloc = Page::new(&dev, 1024, MemoryFlags::MAPABLE)?;
    let mut input = Buffer::<f32, _>::new_uninit(5, UsageFlags::STORAGE_BUFFER, BufferFlags::empty(), MemoryFlags::MAPABLE, &alloc)?;
    let output = Buffer::<f32, _>::new_uninit(5, UsageFlags::STORAGE_BUFFER, BufferFlags::empty(), MemoryFlags::MAPABLE, &alloc)?;

    let mut map = input.map_mut(..)?; 
    map.init_from_slice(&[1f32, 2f32, 3f32, 4f32, 5f32]);
    drop(map);
    let input = unsafe { input.assume_init() };

    let words = include_spv!("target/main.spv");
    //let mut file = std::fs::File::open("target/main.spv")?;
    //let words = read_spv(&mut file)?;
    
    let mut pipeline = ComputeBuilder::new(&dev)
        .binding(DescriptorType::StorageBuffer, 1)
        .binding(DescriptorType::StorageBuffer, 1)
        .build(words)?;
    
    let set = pipeline.sets().first().unwrap();
    let input_desc = set.write_descriptor(&input, 0);
    let output_desc = set.write_descriptor(&output, 0);
    pipeline.sets_mut().update(&[input_desc, output_desc]);
    
    let mut cmds = CommandPool::new(&dev, family, CommandPoolFlags::empty(), 1, CommandBufferLevel::Primary)?;
    let mut cmd_buff = cmds.begin_mut(0, CommandBufferUsage::ONE_TIME_SUBMIT)?;
    cmd_buff.bind_pipeline(PipelineBindPoint::Compute, &pipeline, ..);
    cmd_buff.dispatch(5, 1, 1);
    drop(cmd_buff);

    let mut fence = Fence::new(&dev, FenceFlags::empty())?;
    dev.get_queue(family, 0)?.submitter(&mut fence)
        .add(&cmds, 0..1, None)
        .submit()?;

    fence.wait(None)?;
    let output = unsafe { output.assume_init() };

    let out = &output.map(..)? as &[f32];
    let input = &input.map(..)? as &[f32];
    println!("{out:?}, {input:?}");

    Ok(())
}
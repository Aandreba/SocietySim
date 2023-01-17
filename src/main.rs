#![cfg_attr(docsrs, feature(doc_cfg))]
#![feature(ptr_metadata, rustc_attrs)]

use shared::person::Person;
use vulkan::{
    alloc::{DeviceAllocator, MemoryFlags, Page},
    buffer::{Buffer, BufferFlags, UsageFlags},
    descriptor::DescriptorType,
    device::Device,
    include_spv,
    physical_dev::PhysicalDevice,
    pipeline::{ComputeBuilder, Pipeline},
    pool::{
        CommandBufferUsage, CommandPool, PipelineBindPoint, CommandPoolFlags, CommandBufferLevel,
    },
    queue::{Queue, Fence, FenceFlags},
    utils::u64_to_u32,
    Entry, extension_props,
};
pub mod game;

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
    ($l:literal) => {
        #[allow(unused_unsafe)]
        unsafe { core::ffi::CStr::from_bytes_with_nul_unchecked(concat!($l, "\0").as_bytes()) }
    };
}

fn main() -> anyhow::Result<()> {
    //let _ = unsafe { Entry::builder(1, 0, 0).build_in("/opt/homebrew/Cellar/molten-vk/1.2.1/lib/libMoltenVK.dylib") }?;
    let _ = unsafe {
        Entry::builder(1, 1, 0)
            .build()
    }?;

    #[cfg(debug_assertions)]
    println!("{:#?}", extension_props());

    let phy = PhysicalDevice::first()?;
    let family = phy.families().next().unwrap();
    let (dev, mut queues) = Device::builder(phy)
        .queues(&[1f32]).build()
        .build()?;
    let mut pool = CommandPool::new(&dev, family, CommandPoolFlags::empty(), 1, CommandBufferLevel::Primary)?;

    let alloc = Page::new(&dev, 2048, MemoryFlags::MAPABLE)?;
    let mut people = Buffer::new_uninit(
        5,
        UsageFlags::STORAGE_BUFFER,
        BufferFlags::empty(),
        MemoryFlags::MAPABLE,
        &alloc,
    )?;

    let mut people_map = people.map_mut(..)?;
    for (i, person) in people_map.iter_mut().enumerate() {
        let _ = person.write(Person::new(true, i as u16, 10, 10, 10, 10, 10, 10));
    }
    drop(people_map);
    let mut people = unsafe { people.assume_init() };
    
    let mut main = setup_main(&dev)?;
    call_gpu_main(&mut people, &mut main, &mut pool, &mut queues[0])?;

    let my_people = people.map(..)?;
    for person in my_people.iter() {
        println!("{person:#?}")
    }

    Ok(())
}

fn setup_main<'a> (dev: &'a Device) -> vulkan::Result<Pipeline<'a>> {
    const SHADER: &[u32] = include_spv!("gpu.spv");
    #[cfg(debug_assertions)]
    println!("Shader path: {}", env!("gpu.spv"));

    return ComputeBuilder::new(dev)
        .entry(cstr!("main_cs"))
        .binding(DescriptorType::StorageBuffer, 1)
        .build(SHADER);
}

fn call_gpu_main<A: DeviceAllocator>(
    input: &mut Buffer<Person, A>,
    pipeline: &mut Pipeline,
    pool: &mut CommandPool,
    queue: &mut Queue,
) -> anyhow::Result<()> {
    let set = pipeline.sets().first().unwrap();
    let input_desc = set.write_descriptor(input, 0);
    pipeline.sets_mut().update(&[input_desc]);

    let mut cmd_buff = pool.begin_mut(0, CommandBufferUsage::ONE_TIME_SUBMIT)?;
    cmd_buff.bind_pipeline(PipelineBindPoint::Compute, &pipeline, ..);
    cmd_buff.dispatch(u64_to_u32(input.len()), 1, 1);
    drop(cmd_buff);

    //std::thread::sleep(std::time::Duration::from_secs(2));
    let mut fence = Fence::new(pipeline.device(), FenceFlags::empty())?;
    fence.bind_to(pool, queue, None)?;
    fence.wait(None)?;

    return Ok(());
}

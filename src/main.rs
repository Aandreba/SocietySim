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
    pipeline::ComputeBuilder,
    pool::{
        CommandBufferUsage, CommandPool, PipelineBindPoint,
    },
    queue::{Fence, FenceFlags, Queue},
    utils::u64_to_u32,
    Entry,
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
        unsafe { core::ffi::CStr::from_bytes_with_nul_unchecked(concat!($l, "\0").as_bytes()) }
    };
}

fn main() -> anyhow::Result<()> {
    //let _ = unsafe { Entry::builder(1, 0, 0).build_in("/opt/homebrew/Cellar/molten-vk/1.2.1/lib/libMoltenVK.dylib") }?;
    let _ = unsafe { Entry::builder(1, 1, 0).build() }?;

    let phy = PhysicalDevice::first()?;
    let family = phy.families().next().unwrap();
    let (dev, mut queues) = Device::builder(phy).queues(&[1f32]).build().build()?;

    let alloc = Page::new(&dev, 2048, MemoryFlags::MAPABLE)?;
    let mut people = Buffer::<Person, _>::new_uninit(
        5,
        UsageFlags::STORAGE_BUFFER,
        BufferFlags::empty(),
        MemoryFlags::MAPABLE,
        &alloc,
    )?;

    let mut people_map = people.map_mut(..)?;
    for person in people_map.iter_mut() {
        let _ = person.write(Person {
            is_male: true,
            age: todo!(),
            cordiality_intelligence: todo!(),
            knowledge_finesse: todo!(),
            gullability_health: todo!(),
        });
    }

    Ok(())
}

fn call_gpu_main<A: DeviceAllocator>(
    input: &mut Buffer<Person, A>,
    pool: &mut CommandPool,
    queue: &mut Queue,
) -> anyhow::Result<()> {
    const SHADER: &[u32] = include_spv!("gpu.spv");
    let dev = pool.device();

    let mut pipeline = ComputeBuilder::new(&dev)
        .binding(DescriptorType::StorageBuffer, 1)
        .binding(DescriptorType::StorageBuffer, 1)
        .build(SHADER)?;

    let set = pipeline.sets().first().unwrap();
    let input_desc = set.write_descriptor(&input, 0);
    pipeline.sets_mut().update(&[input_desc]);

    let mut cmd_buff = pool.begin_mut(0, CommandBufferUsage::ONE_TIME_SUBMIT)?;
    cmd_buff.bind_pipeline(PipelineBindPoint::Compute, &pipeline, ..);
    cmd_buff.dispatch(u64_to_u32(input.len()), 1, 1);
    drop(cmd_buff);

    let mut fence = Fence::new(&dev, FenceFlags::empty())?;
    fence.bind_and_wait(pool, queue, None, None);
    return Ok(());
}

#![cfg_attr(docsrs, feature(doc_cfg))]
#![feature(ptr_metadata, rustc_attrs)]

use rand::{thread_rng, Rng, random};
use shared::{person::{Person, PersonStats}, person_event::PersonalEvent, ExternBool, time::GameDuration};
use vulkan::{
    alloc::{DeviceAllocator, MemoryFlags, Page},
    buffer::{Buffer, BufferFlags, UsageFlags},
    descriptor::DescriptorType,
    device::{Device, DeviceRef},
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

const WORDS: &[u32] = include_spv!("gpu.spv");

use crate::game::personal_events::PersonalEvents;
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
    let people = Buffer::from_sized_iter(
        (0..50).into_iter().map(|_| {
            let is_male = random::<bool>();
            let age = random::<u8>() / 100;
            let cordiality = random::<u8>();
            let intelligence = random::<u8>();
            let knowledge = random::<u8>();
            let finesse = random::<u8>();
            let gullability = random::<u8>();
            let health = random::<u8>();

            Person {
                is_male: ExternBool::new(is_male),
                age: GameDuration::from_years(age),
                stats: PersonStats {
                    cordiality,
                    intelligence,
                    knowledge,
                    finesse,
                    gullability,
                    health,
                },
            }
        }),
        UsageFlags::STORAGE_BUFFER,
        BufferFlags::empty(),
        MemoryFlags::MAPABLE,
        &alloc,
    )?;
    let events = Buffer::from_sized_iter(
        [PersonalEvent {
            duration: None,
            chance: PersonStats {
                cordiality: 0.5f32,
                intelligence: 0.25f32,
                knowledge: 0.25f32,
                finesse: 0f32,
                gullability: 0f32,
                health: 0f32,
            },
            effects: PersonStats {
                cordiality: 1,
                intelligence: 0,
                knowledge: 0,
                finesse: 0,
                gullability: 0,
                health: 0,
            },
        }],
        UsageFlags::STORAGE_BUFFER,
        BufferFlags::empty(),
        // MemoryFlags::DEVICE_LOCAL,
        MemoryFlags::MAPABLE,
        &alloc,
    )?;
    
    let mut evt = PersonalEvents::new(&dev, WORDS)?;
    let result = evt.call(&people, &events, &mut pool, queues.first_mut().unwrap())?;

    let result = result.map(..)?;
    println!("{:#?}", &result as &[ExternBool]);

    // let mut main = setup_main(&dev)?;
    // call_gpu_main(&mut people, &mut main, &mut pool, &mut queues[0])?;
    // let my_people = people.map(..)?;
    // for person in my_people.iter() {
    //     println!("{person:#?}")
    // }

    Ok(())
}

fn setup_main<D: Clone + DeviceRef> (dev: D) -> vulkan::Result<Pipeline<D>> {
    #[cfg(debug_assertions)]
    println!("Shader path: {}", env!("gpu.spv"));

    return ComputeBuilder::new(dev)
        .entry(cstr!("main_cs"))
        .binding(DescriptorType::StorageBuffer, 1)
        .build(WORDS);
}

fn call_gpu_main<A: DeviceAllocator, Pi: DeviceRef, Po: DeviceRef>(
    input: &mut Buffer<Person, A>,
    pipeline: &mut Pipeline<Pi>,
    pool: &mut CommandPool<Po>,
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
    fence.bind_to::<_, Pi>(pool, queue, None)?;
    fence.wait(None)?;

    return Ok(());
}

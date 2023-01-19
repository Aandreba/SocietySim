#![cfg_attr(docsrs, feature(doc_cfg))]
#![feature(ptr_metadata, rustc_attrs)]

use shared::{
    person::{PersonStats},
    person_event::PersonalEvent,
    ExternBool,
};
use vulkan::{
    alloc::{MemoryFlags, Page},
    buffer::{Buffer, BufferFlags, UsageFlags},
    device::{Device},
    extension_props, include_spv,
    physical_dev::PhysicalDevice,
    pool::{
        CommandBufferLevel, CommandPool, CommandPoolFlags,
    },
    Entry,
};

const WORDS: &[u32] = include_spv!("gpu.spv");

use crate::game::{generate_people::GeneratePeople, personal_events::PersonalEvents};
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

fn main() -> anyhow::Result<()> {
    //let _ = unsafe { Entry::builder(1, 0, 0).build_in("/opt/homebrew/Cellar/molten-vk/1.2.1/lib/libMoltenVK.dylib") }?;
    let _ = unsafe { Entry::builder(1, 1, 0).build() }?;

    #[cfg(debug_assertions)]
    println!("{:#?}", extension_props());

    let phy = PhysicalDevice::first()?;
    let family = phy.families().next().unwrap();
    let (dev, mut queues) = Device::builder(phy).queues(&[1f32]).build().build()?;
    let mut pool = CommandPool::new(
        &dev,
        family,
        CommandPoolFlags::empty(),
        1,
        CommandBufferLevel::Primary,
    )?;

    let alloc = Page::new(&dev, 2048, MemoryFlags::MAPABLE)?;
    let mut generator = GeneratePeople::new(&dev, WORDS)?;

    let people = generator.generate(
        500,
        UsageFlags::STORAGE_BUFFER,
        BufferFlags::empty(),
        // MemoryFlags::DEVICE_LOCAL,
        MemoryFlags::MAPABLE,
        &alloc,
        &mut pool,
        queues.first_mut().unwrap()
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
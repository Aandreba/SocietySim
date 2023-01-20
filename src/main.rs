#![cfg_attr(docsrs, feature(doc_cfg))]
#![feature(ptr_metadata, rustc_attrs)]

use std::{collections::HashMap, io::BufReader, panic::resume_unwind, path::Path};

use context::Context;
use futures::{pin_mut, stream::FuturesUnordered, FutureExt, Stream, StreamExt, TryStreamExt};
use shared::{person::Person, person_event::PersonalEvent, ExternBool};
use vulkan::{
    alloc::{DeviceAllocator, MemoryFlags, Page},
    buffer::{Buffer, BufferFlags, UsageFlags},
    device::{Device, DeviceRef},
    extension_props, include_spv,
    physical_dev::PhysicalDevice,
    Entry,
};

use crate::game::{generate_people::GeneratePeople, personal_events::PersonalEvents};
pub mod context;
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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    //let _ = unsafe { Entry::builder(1, 0, 0).build_in("/opt/homebrew/Cellar/molten-vk/1.2.1/lib/libMoltenVK.dylib") }?;
    let _ = unsafe { Entry::builder(1, 1, 0).build() }?;

    #[cfg(debug_assertions)]
    println!("{:#?}", extension_props());

    let phy = PhysicalDevice::first()?;
    let (dev, queues) = Device::builder(phy).queues(&[1f32]).build().build()?;
    let alloc = Page::new(&dev, 2048, MemoryFlags::MAPABLE)?;
    let mut ctx = Context::new(&dev, queues.into_iter().next().unwrap())?;

    let people = initialize_population(10_000, &mut ctx, &alloc)?;
    let (_event_names, events) =
        initialize_personal_events("game/personal_events", &mut ctx, &alloc).await?;

    let mut evt = PersonalEvents::new(&dev)?;
    let result = evt.call(&people, &events, &mut ctx)?;

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

#[inline]
fn initialize_population<D: Clone + DeviceRef, A: DeviceAllocator>(
    capacity: u64,
    ctx: &mut Context<D>,
    alloc: A,
) -> vulkan::Result<Buffer<Person, A>> {
    let mut generator = GeneratePeople::new(ctx.owned_device())?;
    return generator.generate(
        capacity,
        UsageFlags::STORAGE_BUFFER,
        BufferFlags::empty(),
        // MemoryFlags::DEVICE_LOCAL,
        MemoryFlags::MAPABLE,
        alloc,
        ctx,
    );
}

#[inline]
async fn initialize_personal_events<
    P: 'static + Send + Clone + AsRef<Path>,
    D: Clone + DeviceRef,
    A: DeviceAllocator,
>(
    path: P,
    ctx: &mut Context<D>,
    alloc: A,
) -> anyhow::Result<(Vec<String>, Buffer<PersonalEvent, A>)> {
    let mut handles = FuturesUnordered::new();
    let mut dir = tokio::fs::read_dir(path.as_ref()).await?;

    while let Some(entry) = dir.next_entry().await? {
        if entry.metadata().await?.is_file() {
            let path = path.clone();
            let task = tokio::task::spawn_blocking(move || {
                let read = BufReader::new(std::fs::File::open(path)?);
                let reader = serde_json::from_reader::<_, HashMap<String, PersonalEvent>>(read)?;
                return anyhow::Ok(reader);
            });

            handles.push(task.map(|x| match x {
                Ok(x) => x,
                Err(e) => resume_unwind(e.into_panic()),
            }));
        }
    }

    let mut names = Vec::new();
    let mut tmp_events = Vec::new();

    while let Some(entries) = handles.try_next().await? {
        names.reserve(entries.len());
        tmp_events.reserve(entries.len());

        for (key, value) in entries {
            names.push(key);
            tmp_events.push(value);
        }
    }

    let mut events = Buffer::<PersonalEvent, _>::new_uninit(
        tmp_events.len() as u64,
        UsageFlags::STORAGE_BUFFER,
        BufferFlags::empty(),
        MemoryFlags::MAPABLE,
        alloc,
    )?;

    let mut map = events.map_mut(..)?;
    map.init_from_slice(&tmp_events);
    drop(map);

    return unsafe { Ok((names, events.assume_init())) };
}

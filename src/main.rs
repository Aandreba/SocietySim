#![cfg_attr(docsrs, feature(doc_cfg))]
#![feature(ptr_metadata, rustc_attrs)]

use futures::{stream::FuturesUnordered, FutureExt, TryStreamExt};
use shared::{person::Person, person_event::PersonalEvent, ExternBool};
use std::{collections::HashMap, io::BufReader, panic::resume_unwind, path::Path};
use tokio::runtime::Runtime;
use vulkan::{
    alloc::{Book, DeviceAllocator, MemoryFlags},
    buffer::{Buffer, BufferFlags, UsageFlags},
    context::{Context},
    physical_dev::PhysicalDevice,
    Entry,
};

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
    let runtime = Runtime::new()?;

    let phy = PhysicalDevice::first()?;
    let ctx = Context::new(phy)?;
    let alloc = Book::new(&ctx, None, None);

    let people = initialize_population(10, &alloc)?;
    let (_event_names, events) = runtime.block_on(initialize_personal_events(
        "game/personal_events",
        &alloc,
    ))?;

    let mut evt = PersonalEvents::new(&ctx)?;
    let result = evt.call(&people, &events)?;

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
fn initialize_population<A: DeviceAllocator>(
    capacity: u64,
    alloc: A,
) -> vulkan::Result<Buffer<Person, A>>
where
    A::Context: Clone,
{
    let mut generator = GeneratePeople::new(alloc.owned_context())?;
    return generator.generate(
        capacity,
        UsageFlags::STORAGE_BUFFER,
        BufferFlags::empty(),
        // MemoryFlags::DEVICE_LOCAL,
        MemoryFlags::MAPABLE,
        alloc,
    );
}

#[inline]
async fn initialize_personal_events<P: 'static + Send + Clone + AsRef<Path>, A: DeviceAllocator>(
    path: P,
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

#[test]
fn disassemble() -> anyhow::Result<()> {
    fn get_path(name: impl AsRef<Path>) -> anyhow::Result<std::path::PathBuf> {
        return Ok([
            "target",
            "spirv-builder",
            "spirv-unknown-vulkan1.1",
            "release",
            "deps",
            "gpu.spvs",
        ]
        .into_iter()
        .fold(std::env::current_dir()?, |x, y| x.join(y))
        .join(name.as_ref().with_extension("spv")));
    }

    fn spirv_cross(name: impl AsRef<Path>) -> anyhow::Result<()> {
        let path = get_path(name.as_ref())?;
        let cmd = std::process::Command::new("spirv-cross")
            .arg("--msl")
            .arg(path)
            .output()?;

        if cmd.status.success() {
            std::fs::write(name.as_ref().with_extension("msl"), cmd.stdout)?;
        } else {
            std::io::copy(&mut cmd.stderr.as_slice(), &mut std::io::stderr())?;
        }
        return Ok(());
    }

    spirv_cross("generate_people")?;
    spirv_cross("compute_personal_event")?;

    return Ok(());
}

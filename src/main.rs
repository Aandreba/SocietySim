#![cfg_attr(docsrs, feature(doc_cfg))]
#![feature(ptr_metadata, rustc_attrs)]

use futures::{future::{try_join}, stream::FuturesUnordered, FutureExt, TryStreamExt};
use population::Population;
use shared::person_event::PersonalEvent;
use std::{collections::HashMap, io::BufReader, panic::resume_unwind, path::Path};
use tokio::runtime::Runtime;
use vulkan::{
    alloc::{Book, DeviceAllocator, MemoryFlags},
    buffer::{Buffer, BufferFlags, UsageFlags},
    context::Context,
    physical_dev::PhysicalDevice,
    r#async::EventRuntime,
    Entry,
};

use crate::game::generate_people::GeneratePeople;
pub mod game;
pub mod menu;
pub mod population;

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
    let mut event_rt = EventRuntime::new(&ctx);

    let population = Population::new(10_000, &alloc)?;

    // let mut evt = PersonalEvents::new(&ctx)?;
    // let result = evt.call(&people, &events)?.wait()?;
    // let result = result.map(..)?;
    // println!("{:#?}", &result as &[ExternBool]);

    // let mut main = setup_main(&dev)?;
    // call_gpu_main(&mut people, &mut main, &mut pool, &mut queues[0])?;
    // let my_people = people.map(..)?;
    // for person in my_people.iter() {
    //     println!("{person:#?}")
    // }

    Ok(())
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
            std::fs::write(name.as_ref().with_extension("c"), cmd.stdout)?;
        } else {
            std::io::copy(&mut cmd.stderr.as_slice(), &mut std::io::stderr())?;
        }
        return Ok(());
    }

    spirv_cross("generate_people")?;
    spirv_cross("compute_personal_event")?;

    return Ok(());
}

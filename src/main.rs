#![cfg_attr(docsrs, feature(doc_cfg))]
#![feature(trait_alias, result_flattening, new_uninit, ptr_metadata, iterator_try_collect, rustc_attrs)]

use shared::population::GenerationOps;
use std::io::Write;
use std::str::FromStr;
use vulkan::{
    alloc::{Book},
    context::Context,
    physical_dev::PhysicalDevice,
    Entry,
};

//pub mod game;
pub mod data;
//pub mod population;
pub mod state;

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
    //let runtime = Runtime::new()?;

    // let phy = PhysicalDevice::first()?;
    // let ctx = Context::new(phy)?;
    // let alloc = Book::new(&ctx, None, None);
    // let mut population = Population::new(200_000, GenerationOps::default(), &alloc)?;
// 
    // loop {
    //     match first_menu(&population)? {
    //         MenuOptions::Stats => {
    //             println!("Population stats: {:#?}", population.count_stats()?);
    //         }
    //         MenuOptions::Exit => return Ok(()),
    //     }
    // }
    todo!()
}

/*
#[repr(u8)]
pub enum MenuOptions {
    Stats = 1,
    Exit = 2,
}

fn first_menu<A: PopulationAllocator>(pops: &Population<A>) -> anyhow::Result<MenuOptions> {
    let mut stdout = std::io::stdout().lock();
    stdout.write_fmt(format_args!("Current population: {}\n", pops.len()))?;
    stdout.write_all(b"1) Stats\n")?;
    stdout.write_all(b"2) Exit\n")?;
    stdout.flush()?;

    loop {
        let mut line = String::new();
        std::io::stdin().read_line(&mut line)?;

        match u8::from_str(line.trim_end()) {
            Ok(x @ 1..=2) => return unsafe { Ok(core::mem::transmute(x)) },
            _ => {
                stdout.write_all(b"Invalid value\n")?;
                stdout.flush()?;
            },
        }
    }
}
*/

#[test]
fn disassemble() -> anyhow::Result<()> {
    fn get_path(name: impl AsRef<std::path::Path>) -> anyhow::Result<std::path::PathBuf> {
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

    fn spirv_cross(name: impl AsRef<std::path::Path>) -> anyhow::Result<()> {
        let path = get_path(name.as_ref())?;
        let cmd = std::process::Command::new("spirv-dis")
            .arg(path)
            .output()?;

        if cmd.status.success() {
            std::fs::write(name.as_ref().with_extension("spirv"), cmd.stdout)?;
        } else {
            std::io::copy(&mut cmd.stderr.as_slice(), &mut std::io::stderr())?;
        }
        return Ok(());
    }

    spirv_cross("generate_people")?;
    spirv_cross("population_mean_stats")?;
    spirv_cross("population_count_stats")?;

    return Ok(());
}

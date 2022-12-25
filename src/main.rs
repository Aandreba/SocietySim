#![cfg_attr(docsrs, feature(doc_cfg))]
#![feature(ptr_metadata)]

use vulkan::{Entry, device::{PhysicalDevice, Device}};

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
    let _ = unsafe { Entry::builder(1, 0, 0).build_in("/opt/homebrew/Cellar/molten-vk/1.2.1/lib/libMoltenVK.dylib") }?;
    
    let phy = PhysicalDevice::first()?;
    let (dev, queues) = Device::builder(&phy)
        .queues(&[1f32]).build()
        .build()?;

    println!("{dev:?}\n{queues:#?}");
    Ok(())
}
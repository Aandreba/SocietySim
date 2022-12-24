use vulkan::Entry;
pub mod vulkan;

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
    let entry = unsafe { Entry::load() }?;
    Ok(())
}
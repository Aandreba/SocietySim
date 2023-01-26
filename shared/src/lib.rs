#![cfg_attr(target_arch = "spirv", no_std, feature(asm_experimental_arch))]
#![feature(portable_simd)]

macro_rules! flat_mod {
    ($($i:ident),+) => {
        $(
            mod $i;
            pub use $i::*;  
        )+
    }
}

pub mod time;
pub mod person;
pub mod simd;
pub mod consts;
pub mod population;
#[cfg(not(target_arch = "spirv"))]
pub mod game_data;
//pub mod sync;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct ExternBool {
    inner: u8
}

impl ExternBool {
    #[inline]
    pub const fn new (v: bool) -> Self {
        return Self { inner: v as u8 }
    }

    #[inline]
    pub const fn get (self) -> bool {
        return self.inner == 1
    }

    #[inline(always)]
    pub fn set (&mut self) {
        self.inner = 1;
    }
}

impl From<bool> for ExternBool {
    #[inline]
    fn from(value: bool) -> Self {
        Self::new(value)
    }
}

impl Into<bool> for ExternBool {
    #[inline]
    fn into(self) -> bool {
        self.get()
    }
}

#[cfg(not(target_arch = "spirv"))]
impl core::fmt::Debug for ExternBool {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(&self.get(), f)
    }
}
#![no_std]

pub mod person;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
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
        return unsafe { core::mem::transmute(self.inner) }
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
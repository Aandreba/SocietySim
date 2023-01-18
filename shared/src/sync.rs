use core::cell::UnsafeCell;

const UNLOCKED: u8 = 0;
#[allow(unused)]
const LOCKED: u8 = 1;

#[repr(transparent)]
pub struct ExternMutex {
    inner: UnsafeCell<u8>,
}

impl ExternMutex {
    #[inline]
    pub const fn new() -> Self {
        return Self { inner: UnsafeCell::new(UNLOCKED) };
    }

    #[cfg(not(target_arch = "spirv"))]
    pub fn is_unlocked (&self) -> bool {
        return unsafe { *self.inner.get() == UNLOCKED }
    }

    #[cfg(target_arch = "spirv")]
    #[inline]
    pub fn lock(&self) {
        loop {
            match spirv_std::arch::atomic_compare_exchange::<
                u8,
                1, // Device
                0x8, // AcqRel
                0x2, // Acquire
            >(unsafe { &mut *self.inner.get() }, LOCKED, UNLOCKED) {
                UNLOCKED => break,  
                _ => {},
            }
        }
    }

    #[cfg(target_arch = "spirv")]
    #[inline]
    pub unsafe fn unlock (&self) {
        spirv_std::arch::atomic_store::<
            u8,
            1, // Device
            0x4 // Release
        >(unsafe { &mut *self.inner.get() }, UNLOCKED);
    }
}

unsafe impl Send for ExternMutex {}
unsafe impl Sync for ExternMutex {}
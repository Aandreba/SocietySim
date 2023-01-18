use core::ops::Mul;

pub struct f32x4 {
    #[cfg(target_arch = "spriv")]
    inner: glam::Vec4,
    #[cfg(not(target_arch = "spirv"))]
    inner: core::simd::f32x4
}

impl f32x4 {
    #[inline]
    pub const fn from_array (inner: [f32; 4]) -> Self {
        cfg_if::cfg_if! {
            if #[cfg(target_arch = "spriv")] {
                return Self { inner: glam::Vec4::from_array(inner) }
            } else {
                return Self { inner: core::simd::f32x4::from_array(inner) }
            }
        }
    }

    #[cfg(target_arch = "spriv")]
    pub const fn from_glam (inner: glam::Vec4) -> Self {
        return Self { inner }
    }
}

impl Mul for f32x4 {
    type Output = f32x4;

    #[cfg(target_arch = "spriv")]
    #[inline]
    fn mul(mut self, rhs: Self) -> Self::Output {
        unsafe {
            asm! {
                "{out} OpFMul _ {lhs} {rhs}",
                out = out(reg) self.inner,
                lhs = in(reg) self.inner,
                rhs = in(reg) rhs.inner
            }
            return self
        }
    }

    #[cfg(not(target_arch = "spriv"))]
    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        return Self { inner: self.inner * rhs.inner }
    }
}
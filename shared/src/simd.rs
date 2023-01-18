use core::ops::Mul;
#[cfg(target_arch = "spirv")]
use core::arch::asm;


/* SINGLE PRECISION */
#[derive(Clone, Copy)]
#[allow(non_camel_case_types)]
pub struct f32x2 {
    #[cfg(not(target_arch = "spirv"))]
    inner: core::simd::f32x2,
    #[cfg(target_arch = "spirv")]
    inner: spirv_std::glam::Vec2
}

impl f32x2 {
    #[inline]
    pub const fn from_array (inner: [f32; 2]) -> Self {
        #[cfg(not(target_arch = "spirv"))]
        return Self { inner: core::simd::f32x2::from_array(inner) };
        #[cfg(target_arch = "spirv")]
        return Self { inner: spirv_std::glam::Vec2::from_array(inner) };
    }

    #[inline]
    pub fn dot (self, rhs: Self) -> f32 {
        return (self * rhs).reduce_sum()
    }

    pub fn reduce_sum (self) -> f32 {
        #[cfg(not(target_arch = "spirv"))]
        return core::simd::SimdFloat::reduce_sum(self.inner);
        #[cfg(target_arch = "spirv")]
        return self.inner.x + self.inner.y
    }
}

impl Mul for f32x2 {
    type Output = f32x2;

    #[cfg(target_arch = "spirv")]
    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        unsafe {
            let mut inner = spirv_std::glam::Vec2::default();
            asm! {
                "%lhs = OpLoad _ {lhs}",
                "%rhs = OpLoad _ {rhs}",
                "%result = OpFMul _ %lhs %rhs",
                "OpStore {result} %result",
                lhs = in(reg) &self.inner,
                rhs = in(reg) &rhs.inner,
                result = in(reg) &mut inner
            }
            return Self { inner }
        }
    }

    #[cfg(not(target_arch = "spirv"))]
    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        return Self { inner: self.inner * rhs.inner }
    }
}

#[derive(Clone, Copy)]
#[allow(non_camel_case_types)]
pub struct f32x3 {
    #[cfg(not(target_arch = "spirv"))]
    inner: core::simd::f32x4,
    #[cfg(target_arch = "spirv")]
    inner: spirv_std::glam::Vec3
}

impl f32x3 {
    #[cfg(not(target_arch = "spirv"))]
    #[inline]
    pub const fn from_array ([x, y, z]: [f32; 3]) -> Self {
        return Self { inner: core::simd::f32x4::from_array([x, y, z, 0f32]) };
    }

    #[cfg(target_arch = "spirv")]
    #[inline]
    pub const fn from_array (inner: [f32; 3]) -> Self {
        return Self { inner: spirv_std::glam::Vec3::from_array(inner) };
    }

    #[inline]
    pub fn dot (self, rhs: Self) -> f32 {
        return (self * rhs).reduce_sum()
    }

    pub fn reduce_sum (self) -> f32 {
        #[cfg(not(target_arch = "spirv"))]
        return core::simd::SimdFloat::reduce_sum(self.inner);
        #[cfg(target_arch = "spirv")]
        return self.inner.x + self.inner.y + self.inner.z
    }
}

impl Mul for f32x3 {
    type Output = f32x3;

    #[cfg(target_arch = "spirv")]
    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        unsafe {
            let mut inner = spirv_std::glam::Vec3::default();
            asm! {
                "%lhs = OpLoad _ {lhs}",
                "%rhs = OpLoad _ {rhs}",
                "%result = OpFMul _ %lhs %rhs",
                "OpStore {result} %result",
                lhs = in(reg) &self.inner,
                rhs = in(reg) &rhs.inner,
                result = in(reg) &mut inner
            }
            return Self { inner }
        }
    }

    #[cfg(not(target_arch = "spirv"))]
    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        return Self { inner: self.inner * rhs.inner }
    }
}

#[derive(Clone, Copy)]
#[allow(non_camel_case_types)]
pub struct f32x4 {
    #[cfg(not(target_arch = "spirv"))]
    inner: core::simd::f32x4,
    #[cfg(target_arch = "spirv")]
    inner: spirv_std::glam::Vec4
}

impl f32x4 {
    #[inline]
    pub const fn from_array (inner: [f32; 4]) -> Self {
        #[cfg(not(target_arch = "spirv"))]
        return Self { inner: core::simd::f32x4::from_array(inner) };
        #[cfg(target_arch = "spirv")]
        return Self { inner: spirv_std::glam::Vec4::from_array(inner) };
    }

    #[inline]
    pub fn dot (self, rhs: Self) -> f32 {
        return (self * rhs).reduce_sum()
    }

    pub fn reduce_sum (self) -> f32 {
        #[cfg(not(target_arch = "spirv"))]
        return core::simd::SimdFloat::reduce_sum(self.inner);
        #[cfg(target_arch = "spirv")]
        return self.inner.x + self.inner.y + self.inner.z + self.inner.w
    }
}

impl Mul for f32x4 {
    type Output = f32x4;

    #[cfg(target_arch = "spirv")]
    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        unsafe {
            let mut inner = spirv_std::glam::Vec4::default();
            asm! {
                "%lhs = OpLoad _ {lhs}",
                "%rhs = OpLoad _ {rhs}",
                "%result = OpFMul _ %lhs %rhs",
                "OpStore {result} %result",
                lhs = in(reg) &self.inner,
                rhs = in(reg) &rhs.inner,
                result = in(reg) &mut inner
            }
            return Self { inner }
        }
    }

    #[cfg(not(target_arch = "spirv"))]
    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        return Self { inner: self.inner * rhs.inner }
    }
}

/* DOUBLE PRECISION */
#[derive(Clone, Copy)]
#[allow(non_camel_case_types)]
pub struct f64x2 {
    #[cfg(not(target_arch = "spirv"))]
    inner: core::simd::f64x2,
    #[cfg(target_arch = "spirv")]
    inner: spirv_std::glam::DVec2
}

impl f64x2 {
    #[inline]
    pub const fn from_array (inner: [f64; 2]) -> Self {
        #[cfg(not(target_arch = "spirv"))]
        return Self { inner: core::simd::f64x2::from_array(inner) };
        #[cfg(target_arch = "spirv")]
        return Self { inner: spirv_std::glam::DVec2::from_array(inner) };
    }

    #[inline]
    pub fn dot (self, rhs: Self) -> f64 {
        return (self * rhs).reduce_sum()
    }

    pub fn reduce_sum (self) -> f64 {
        #[cfg(not(target_arch = "spirv"))]
        return core::simd::SimdFloat::reduce_sum(self.inner);
        #[cfg(target_arch = "spirv")]
        return self.inner.x + self.inner.y
    }
}

impl Mul for f64x2 {
    type Output = f64x2;

    #[cfg(target_arch = "spirv")]
    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        unsafe {
            let mut inner = spirv_std::glam::DVec2::default();
            asm! {
                "%lhs = OpLoad _ {lhs}",
                "%rhs = OpLoad _ {rhs}",
                "%result = OpFMul _ %lhs %rhs",
                "OpStore {result} %result",
                lhs = in(reg) &self.inner,
                rhs = in(reg) &rhs.inner,
                result = in(reg) &mut inner
            }
            return Self { inner }
        }
    }

    #[cfg(not(target_arch = "spirv"))]
    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        return Self { inner: self.inner * rhs.inner }
    }
}

#[derive(Clone, Copy)]
#[allow(non_camel_case_types)]
pub struct f64x3 {
    #[cfg(not(target_arch = "spirv"))]
    inner: core::simd::f64x4,
    #[cfg(target_arch = "spirv")]
    inner: spirv_std::glam::DVec3
}

impl f64x3 {
    #[cfg(not(target_arch = "spirv"))]
    #[inline]
    pub const fn from_array ([x, y, z]: [f64; 3]) -> Self {
        return Self { inner: core::simd::f64x4::from_array([x, y, z, 0f64]) };
    }

    #[cfg(target_arch = "spirv")]
    #[inline]
    pub const fn from_array (inner: [f64; 3]) -> Self {
        return Self { inner: spirv_std::glam::DVec3::from_array(inner) };
    }

    #[inline]
    pub fn dot (self, rhs: Self) -> f64 {
        return (self * rhs).reduce_sum()
    }

    pub fn reduce_sum (self) -> f64 {
        #[cfg(not(target_arch = "spirv"))]
        return core::simd::SimdFloat::reduce_sum(self.inner);
        #[cfg(target_arch = "spirv")]
        return self.inner.x + self.inner.y + self.inner.z
    }
}

impl Mul for f64x3 {
    type Output = f64x3;

    #[cfg(target_arch = "spirv")]
    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        unsafe {
            let mut inner = spirv_std::glam::DVec3::default();
            asm! {
                "%lhs = OpLoad _ {lhs}",
                "%rhs = OpLoad _ {rhs}",
                "%result = OpFMul _ %lhs %rhs",
                "OpStore {result} %result",
                lhs = in(reg) &self.inner,
                rhs = in(reg) &rhs.inner,
                result = in(reg) &mut inner
            }
            return Self { inner }
        }
    }

    #[cfg(not(target_arch = "spirv"))]
    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        return Self { inner: self.inner * rhs.inner }
    }
}
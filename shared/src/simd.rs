use core::ops::*;
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
    
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "spirv")] {
            #[inline]
            pub fn x (self) -> f32 {
                return self.inner.x
            }

            #[inline]
            pub fn y (self) -> f32 {
                return self.inner.y
            }

            #[inline]
            pub fn x_mut (&mut self) -> &mut f32 {
                return &mut self.inner.x
            }

            #[inline]
            pub fn y_mut (&mut self) -> &mut f32 {
                return &mut self.inner.y
            }
        } else {
            #[inline]
            pub fn x (self) -> f32 {
                return self.inner[0]
            }

            #[inline]
            pub fn y (self) -> f32 {
                return self.inner[1]
            }

            #[inline]
            pub fn x_mut (&mut self) -> &mut f32 {
                return &mut self.inner[0]
            }

            #[inline]
            pub fn y_mut (&mut self) -> &mut f32 {
                return &mut self.inner[1]
            }
        }
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

impl Add for f32x2 {
    type Output = f32x2;

    #[cfg(target_arch = "spirv")]
    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        unsafe {
            let mut inner = spirv_std::glam::Vec2::default();
            asm! {
                "%lhs = OpLoad _ {lhs}",
                "%rhs = OpLoad _ {rhs}",
                "%result = OpFAdd _ %lhs %rhs",
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
    fn add(self, rhs: Self) -> Self::Output {
        return Self { inner: self.inner + rhs.inner }
    }
}

impl Sub for f32x2 {
    type Output = f32x2;

    #[cfg(target_arch = "spirv")]
    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        unsafe {
            let mut inner = spirv_std::glam::Vec2::default();
            asm! {
                "%lhs = OpLoad _ {lhs}",
                "%rhs = OpLoad _ {rhs}",
                "%result = OpFSub _ %lhs %rhs",
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
    fn sub(self, rhs: Self) -> Self::Output {
        return Self { inner: self.inner + rhs.inner }
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

impl Mul<f32> for f32x2 {
    type Output = f32x2;

    #[cfg(target_arch = "spirv")]
    #[inline]
    fn mul(self, rhs: f32) -> Self::Output {
        unsafe {
            let mut inner = spirv_std::glam::Vec2::default();
            asm! {
                "%float = OpTypeFloat 32",
                "%vec = OpTypeVector %float 2",
                "%lhs = OpLoad _ {lhs}",
                "%rhs = OpLoad _ {rhs}",
                "%result = OpVectorTimesScalar %vec %lhs %rhs",
                "OpStore {result} %result",
                lhs = in(reg) &self.inner,
                rhs = in(reg) &rhs,
                result = in(reg) &mut inner
            }
            return Self { inner }
        }
    }

    #[cfg(not(target_arch = "spirv"))]
    #[inline]
    fn mul(self, rhs: f32) -> Self::Output {
        return Self { inner: self.inner * core::simd::f32x2::splat(rhs) }
    }
}

impl Mul<f32x2> for f32 {
    type Output = f32x2;

    #[inline]
    fn mul(self, rhs: f32x2) -> Self::Output {
        rhs * self
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

impl Add for f32x3 {
    type Output = f32x3;

    #[cfg(target_arch = "spirv")]
    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        unsafe {
            let mut inner = spirv_std::glam::Vec3::default();
            asm! {
                "%lhs = OpLoad _ {lhs}",
                "%rhs = OpLoad _ {rhs}",
                "%result = OpFAdd _ %lhs %rhs",
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
    fn add(self, rhs: Self) -> Self::Output {
        return Self { inner: self.inner + rhs.inner }
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

impl Add for f32x4 {
    type Output = f32x4;

    #[cfg(target_arch = "spirv")]
    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        unsafe {
            let mut inner = spirv_std::glam::Vec4::default();
            asm! {
                "%lhs = OpLoad _ {lhs}",
                "%rhs = OpLoad _ {rhs}",
                "%result = OpFAdd _ %lhs %rhs",
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
    fn add(self, rhs: Self) -> Self::Output {
        return Self { inner: self.inner + rhs.inner }
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
cfg_if::cfg_if! {
    if #[cfg(target_feature = "Float64")] {
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
    }
}

/* UNSIGNED INTEGER */
#[derive(Clone, Copy, Default)]
#[allow(non_camel_case_types)]
pub struct u32x4 {
    #[cfg(not(target_arch = "spirv"))]
    inner: core::simd::u32x4,
    #[cfg(target_arch = "spirv")]
    inner: spirv_std::glam::UVec4
}

impl u32x4 {
    #[inline]
    pub const fn from_array (inner: [u32; 4]) -> Self {
        #[cfg(not(target_arch = "spirv"))]
        return Self { inner: core::simd::u32x4::from_array(inner) };
        #[cfg(target_arch = "spirv")]
        return Self { inner: spirv_std::glam::UVec4::from_array(inner) };
    }

    cfg_if::cfg_if! {
        if #[cfg(target_arch = "spirv")] {
            #[inline]
            pub fn x (self) -> u32 {
                return self.inner.x
            }

            #[inline]
            pub fn y (self) -> u32 {
                return self.inner.y
            }

            #[inline]
            pub fn z (self) -> u32 {
                return self.inner.z
            }

            #[inline]
            pub fn w (self) -> u32 {
                return self.inner.w
            }

            #[inline]
            pub fn x_mut (&mut self) -> &mut u32 {
                return &mut self.inner.x
            }

            #[inline]
            pub fn y_mut (&mut self) -> &mut u32 {
                return &mut self.inner.y
            }

            #[inline]
            pub fn z_mut (&mut self) -> &mut u32 {
                return &mut self.inner.z
            }

            #[inline]
            pub fn w_mut (&mut self) -> &mut u32 {
                return &mut self.inner.w
            }
        } else {
            #[inline]
            pub fn x (self) -> u32 {
                return self.inner[0]
            }

            #[inline]
            pub fn y (self) -> u32 {
                return self.inner[1]
            }

            #[inline]
            pub fn z (self) -> u32 {
                return self.inner[2]
            }

            #[inline]
            pub fn w (self) -> u32 {
                return self.inner[3]
            }

            #[inline]
            pub fn x_mut (&mut self) -> &mut u32 {
                return &mut self.inner[0]
            }

            #[inline]
            pub fn y_mut (&mut self) -> &mut u32 {
                return &mut self.inner[1]
            }

            #[inline]
            pub fn z_mut (&mut self) -> &mut u32 {
                return &mut self.inner[2]
            }

            #[inline]
            pub fn w_mut (&mut self) -> &mut u32 {
                return &mut self.inner[4]
            }
        }
    }

    #[inline]
    pub fn dot (self, rhs: Self) -> u32 {
        return (self * rhs).reduce_sum()
    }

    pub fn reduce_sum (self) -> u32 {
        #[cfg(not(target_arch = "spirv"))]
        return core::simd::SimdUint::reduce_sum(self.inner);
        #[cfg(target_arch = "spirv")]
        return self.inner.x + self.inner.y + self.inner.z + self.inner.w
    }
}

impl Add for u32x4 {
    type Output = u32x4;

    // #[cfg(target_arch = "spirv")]
    // #[inline]
    // fn add(self, rhs: Self) -> Self::Output {
    //     unsafe {
    //         let mut inner = spirv_std::glam::UVec4::default();
    //         asm! {
    //             "%lhs = OpLoad _ {lhs}",
    //             "%rhs = OpLoad _ {rhs}",
    //             "%result = OpIAdd typeof{result} %lhs %rhs",
    //             "OpStore {result} %result",
    //             lhs = in(reg) &self.inner,
    //             rhs = in(reg) &rhs.inner,
    //             result = in(reg) &mut inner
    //         }
    //         return Self { inner }
    //     }
    // }

    // #[cfg(not(target_arch = "spirv"))]
    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        return Self { inner: self.inner + rhs.inner }
    }
}

impl AddAssign for u32x4 {
    #[inline]
    fn add_assign (&mut self, rhs: Self) {
        *self = *self + rhs
    }
}

impl Mul for u32x4 {
    type Output = u32x4;

    // #[cfg(target_arch = "spirv")]
    // #[inline]
    // fn mul(self, rhs: Self) -> Self::Output {
    //     unsafe {
    //         let mut inner = spirv_std::glam::UVec4::default();
    //         asm! {
    //             "%lhs = OpLoad _ {lhs}",
    //             "%rhs = OpLoad _ {rhs}",
    //             "%result = OpIMul typeof{result} %lhs %rhs",
    //             "OpStore {result} %result",
    //             lhs = in(reg) &self.inner,
    //             rhs = in(reg) &rhs.inner,
    //             result = in(reg) &mut inner
    //         }
    //         return Self { inner }
    //     }
    // }

    // #[cfg(not(target_arch = "spirv"))]
    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        return Self { inner: self.inner * rhs.inner }
    }
}

// TODO FIX INLINE ASM BUG 
impl BitXor for u32x4 {
    type Output = u32x4;

    // #[cfg(target_arch = "spirv")]
    // #[inline]
    // fn bitxor(self, rhs: Self) -> Self::Output {
    //     unsafe {
    //         let mut inner = spirv_std::glam::UVec4::default();
    //         asm! {
    //             "%lhs = OpLoad _ {lhs}",
    //             "%rhs = OpLoad _ {rhs}",
    //             "%result = OpBitwiseXor typeof{result} %lhs %rhs",
    //             "OpStore {result} %result",
    //             lhs = in(reg) &self.inner,
    //             rhs = in(reg) &rhs.inner,
    //             result = in(reg) &mut inner
    //         }
    //         return Self { inner }
    //     }
    // }

    //#[cfg(not(target_arch = "spirv"))]
    #[inline]
    fn bitxor(self, rhs: Self) -> Self::Output {
        return Self { inner: self.inner ^ rhs.inner }
    }
}

impl BitXorAssign for u32x4 {
    #[inline]
    fn bitxor_assign (&mut self, rhs: Self) {
        *self = *self ^ rhs
    }
}
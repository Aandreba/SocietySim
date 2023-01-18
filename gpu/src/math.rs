#[cfg(target_arch = "spirv")]
use core::arch::asm;
use spirv_std::num_traits::NumOps;

pub trait CheckedArith: Sized + NumOps {
    #[inline]
    fn add (self, rhs: Self) -> Self {
        #[cfg(debug_assertions)]
        return self.checked_add(rhs).expect("addition overflow");
        #[cfg(not(debug_assertions))]
        return self + rhs
    }

    #[inline]
    fn mul (self, rhs: Self) -> Self {
        #[cfg(debug_assertions)]
        return self.checked_mul(rhs).expect("addition overflow");
        #[cfg(not(debug_assertions))]
        return self + rhs
    }


    fn checked_add (self, rhs: Self) -> Option<Self>;
    fn checked_mul (self, rhs: Self) -> Option<Self>;
}

macro_rules! impl_check {
    ($($t:ty as $uint:ty),+) => {
        $(
            impl CheckedArith for $t {
                #[inline]
                fn checked_add (self, rhs: Self) -> Option<Self> {
                    let result = self.wrapping_add(rhs);
                    if (result as $uint) > (self as $uint) { return None }
                    return Some(result)
                }

                #[inline]
                fn checked_mul (self, rhs: Self) -> Option<Self> {
                    let result = self.wrapping_mul(rhs);
                    if (result as $uint) > (self as $uint) { return None }
                    return Some(result)
                }
            }
        )+
    };
}

impl_check! {
    u8 as u8, u16 as u16, u32 as u32,
    i8 as u8, i16 as u16, i32 as u32
}

pub trait FloatMath {
    fn fract (self) -> Self;
    fn sin (self) -> Self;
    fn cos (self) -> Self; 
}

// Ids from https://registry.khronos.org/SPIR-V/specs/unified1/GLSL.std.450.html
macro_rules! impl_math {
    ($($t:ty),+) => {
        $(
            impl FloatMath for $t {
                impl_mono! {
                    fract as 10,
                    sin as 13,
                    cos as 14
                }
            }
        )+
    };
}

macro_rules! impl_mono {
    ($($fn:ident as $name:literal),+) => {
        $(
            #[cfg(target_arch = "spirv")]
            #[inline]
            fn $fn (self) -> Self {
                unsafe {
                    const OP: u32 = $name;
                    let mut result = Self::default();
                    asm! {
                        "%glsl = OpExtInstImport \"GLSL.std.450\"",
                        "%this = OpLoad _ {this}",
                        "%result = OpExtInst typeof*{result} %glsl {op} %this",
                        "OpStore {result} %result",
                        this = in(reg) &self,
                        result = in(reg) &mut result,
                        op = const OP
                    };
                    return result
                }
            }

            #[cfg(not(target_arch = "spirv"))]
            #[inline]
            fn $fn (self) -> Self {
                return Self::$fn(self)
            }
        )+
    };
}

impl_math! { f32, f64 }
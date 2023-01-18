#[cfg(target_arch = "spirv")]
use core::arch::asm;

pub trait FloatMath {
    fn fract (self) -> Self;
    fn sin (self) -> Self;   
}

// Ids from https://registry.khronos.org/SPIR-V/specs/unified1/GLSL.std.450.html
macro_rules! impl_math {
    ($($t:ty),+) => {
        $(
            impl FloatMath for $t {
                impl_mono! {
                    fract as 10,
                    sin as 13
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
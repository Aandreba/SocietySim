#[cfg(target_arch = "spirv")]
use core::arch::asm;
use spirv_std::num_traits::NumOps;

pub trait CheckedArith: Sized + NumOps {
    fn overflowing_add (self, rhs: Self) -> (Self, bool);

    #[inline]
    fn checked_add (self, rhs: Self) -> Option<Self> {
        match self.overflowing_add(rhs) {
            (_, true) => None,
            (x, _) => Some(x)
        }
    }
}

macro_rules! impl_check {
    ($($t:ty as $uint:ty),+) => {
        $(
            impl CheckedArith for $t {
                #[inline]
                fn overflowing_add (self, rhs: Self) -> (Self, bool) {
                    let result = self.wrapping_add(rhs);
                    return (result, (result as $uint) > (self as $uint))
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
    /// Returns the fractional part of `self`.
    ///
    /// # Examples
    ///
    /// ```
    /// let x = 3.6_f32;
    /// let y = -3.6_f32;
    /// let abs_difference_x = (x.fract() - 0.6).abs();
    /// let abs_difference_y = (y.fract() - (-0.6)).abs();
    ///
    /// assert!(abs_difference_x <= f32::EPSILON);
    /// assert!(abs_difference_y <= f32::EPSILON);
    /// ```
    fn fract (self) -> Self;

    /// Computes the sine of a number (in radians).
    ///
    /// # Examples
    ///
    /// ```
    /// let x = std::f32::consts::FRAC_PI_2;
    ///
    /// let abs_difference = (x.sin() - 1.0).abs();
    ///
    /// assert!(abs_difference <= f32::EPSILON);
    /// ```
    fn sin (self) -> Self;

    /// Computes the cosine of a number (in radians).
    ///
    /// # Examples
    ///
    /// ```
    /// let x = 2.0 * std::f32::consts::PI;
    ///
    /// let abs_difference = (x.cos() - 1.0).abs();
    ///
    /// assert!(abs_difference <= f32::EPSILON);
    /// ```
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
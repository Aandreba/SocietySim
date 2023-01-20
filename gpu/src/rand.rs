use shared::simd::{f32x2, u32x4};
use crate::math::*;

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Random {
    inner: u32x4
}

impl Random {
    #[inline]
    pub const fn from_seed (seed: [u32; 4]) -> Self {
        return Self { inner: u32x4::from_array(seed) }
    }

    #[inline]
    pub fn from_entropy (seed: [u32; 4], point: f32x2) -> Self {
        let mut this = Self::from_seed(seed);
        this.apply_entropy(point);
        return this
    }

    #[inline]
    pub fn apply_entropy (&mut self, point: f32x2) {
        const ALPHA: f32x2 = f32x2::from_array([12.9898, 78.233]);
        const BETA: f32 = 43758.5453123;
        const MAX: f32 = u32::MAX as f32;

        let entropy = FloatMath::fract(FloatMath::sin(ALPHA.dot(point)) * BETA);
        let entropy = (entropy * MAX) as u32;
        self.inner += u32x4::from_array([entropy; 4]);
    }

    /// This is the jump function for the generator. It is equivalent to 2^64 calls to next().
    /// It can be used to generate 2^64 non-overlapping subsequences for parallel computations.
    pub fn jump (&mut self) -> u32 {
        const JUMP: [u32; 4] = [0x8764000b, 0xf542d2d3, 0x6fa035c3, 0x77f2db5b];

        let mut result = u32x4::default();
        for j in JUMP {
            for b in 0..32 {
                if (j & 1u32) << b != 0 {
                    result ^= self.inner
                }
                let _ = self.next_u32();
            }
        }

        todo!()
    }
}

impl Random {
    #[inline]
    pub fn next_bool (&mut self) -> bool {
        return self.next_u32() & 1 == 1
    }
    
    #[inline]
    pub fn next_u8 (&mut self) -> u8 {
        return self.next_u32() as u8
    }

    #[inline]
    pub fn next_u16 (&mut self) -> u16 {
        return self.next_u32() as u16
    }

    #[inline]
    pub fn next_u32 (&mut self) -> u32 {
        let result = (self.inner.x() + self.inner.w()).rotate_left(7) + self.inner.x();
        let t = self.inner.y() << 9;
        *self.inner.z_mut() ^= self.inner.x();
        *self.inner.w_mut() ^= self.inner.y();
        *self.inner.y_mut() ^= self.inner.z();
        *self.inner.x_mut() ^= self.inner.w();
        *self.inner.z_mut() ^= t;
        *self.inner.w_mut() = self.inner.w().rotate_left(11);
        return result
    }

    #[inline]
    pub fn next_u64 (&mut self) -> u64 {
        return (self.next_u32() as u64) << 32 | (self.next_u32() as u64)
    }

    #[inline]
    pub fn next_f32 (&mut self) -> f32 {
        const SIZE: u32 = 32;
        const PRECISION: u32 = f32::MANTISSA_DIGITS;
        const SCALE: f32 = 1.0 / (((1 as u32) << PRECISION) as f32);

        let mut value = self.next_u32();
        value >>= SIZE - PRECISION;
        return SCALE * ((value + 1) as f32)
    }

    #[cfg(target_feature = "Float64")]
    #[inline]
    pub fn next_f64 (&mut self) -> f64 {
        const SIZE: u32 = 32;
        const PRECISION: u32 = f32::MANTISSA_DIGITS;
        const SCALE: f64 = 1.0 / (((1 as u64) << PRECISION) as f64);

        let mut value = self.next_u64();
        value >>= SIZE - PRECISION;
        return SCALE * ((value + 1) as f64)
    }
}
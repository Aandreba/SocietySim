use core::ops::AddAssign;
use shared::simd::{u32x4, f32x2};
use crate::math::{CheckedArith, FloatMath};

#[derive(Clone, Copy)]
pub struct Random {
    inner: u32x4,
    gaussian: f32,
    has_next: bool
}

impl Random {
    #[inline]
    pub const fn from_seed (seed: [u32; 4]) -> Self {
        return Self {
            inner: u32x4::from_array(seed),
            gaussian: 0.0,
            has_next: false
        }
    }

    #[inline]
    pub fn from_entropy (seed: [u32; 4], point: u32) -> Self {
        let mut this = Self::from_seed(seed);
        this.apply_entropy(point);
        return this
    }

    #[inline]
    pub fn apply_entropy (&mut self, point: u32) {
        match CheckedArith::overflowing_add(self.inner.x(), point) {
            (x, false) => *self.inner.x_mut() = x,
            (point, true) => {
                *self.inner.x_mut() = 0;
                match CheckedArith::overflowing_add(self.inner.y(), point) {
                    (x, false) => *self.inner.y_mut() = x,
                    (point, true) => {
                        *self.inner.y_mut() = 0;
                        match CheckedArith::overflowing_add(self.inner.z(), point) {
                            (x, false) => *self.inner.z_mut() = x,
                            (point, true) => {
                                *self.inner.z_mut() = 0;
                                self.inner.w_mut().add_assign(point);
                            }
                        }
                    }
                }
            }
        }
        
        self.jump();
    }

    /// This is the jump function for the generator. It is equivalent to 2^64 calls to next().
    /// It can be used to generate 2^64 non-overlapping subsequences for parallel computations.
    pub fn jump (&mut self) {
        const JUMP: [u32; 4] = [0x8764000b, 0xf542d2d3, 0x6fa035c3, 0x77f2db5b];

        let mut result = u32x4::default();
        for i in 0..JUMP.len() {
            let j = JUMP[i];
            for b in 0..32 {
                if (j & 1u32) << b != 0 {
                    result ^= self.inner
                }
                let _ = self.next_u32();
            }
        }

        self.inner = result;
    }
}

impl Random {
    #[inline]
    pub fn next_bool (&mut self) -> bool {
        return (self.next_u32() as i32) < 0
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
        const SIZE: u32 = 64;
        const PRECISION: u32 = f64::MANTISSA_DIGITS;
        const SCALE: f64 = 1.0 / (((1 as u64) << PRECISION) as f64);

        let mut value = self.next_u64();
        value >>= SIZE - PRECISION;
        return SCALE * ((value + 1) as f64)
    }
}

impl Random {
    pub fn next_gaussian (&mut self, std: f32, mean: f32) -> f32 {
        if self.has_next {
            self.has_next = false;
            return self.gaussian
        }

        let mut v: f32x2;
        let mut s: f32;

        loop {
            v = 2.0 * f32x2::from_array([self.next_f32(), self.next_f32()]) - f32x2::from_array([1.0; 2]);
            s = v.dot(v);
            if s < 1.0 && s != 0.0 { break }
        }

        let multiplier = FloatMath::sqrt(-2.0 * FloatMath::ln(s) / s);
        v = v * multiplier;

        self.has_next = true;
        self.gaussian = v.y();
        return std * v.x() + mean
    }
}
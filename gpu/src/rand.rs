use shared::simd::{f32x2, f64x2, f32x3, f64x3};
use crate::math::*;

pub trait Random2 {
    fn generate(self, y: Self) -> Self;
}

impl Random2 for f32 {
    #[inline]
    fn generate(self, y: Self) -> Self {
        const ALPHA: f32x2 = f32x2::from_array([12.9898, 78.233]);
        const BETA: f32 = 43758.5453123;
        FloatMath::fract(FloatMath::sin(ALPHA.dot(f32x2::from_array([self, y]))) * BETA)
    }
}

impl Random2 for f64 {
    #[inline]
    fn generate(self, y: Self) -> Self {
        const ALPHA: f64x2 = f64x2::from_array([12.9898, 78.233]);
        const BETA: f64 = 43758.5453123;
        FloatMath::fract(FloatMath::sin(ALPHA.dot(f64x2::from_array([self, y]))) * BETA)
    }
}

pub trait Random3 {
    fn generate(self, y: Self, z: Self) -> Self;
}

impl Random3 for f32 {
    #[inline]
    fn generate(self, y: Self, z: Self) -> Self {
        const ALPHA: f32x3 = f32x3::from_array([12.9898, 78.233, 45.6114]);
        const BETA: f32 = 43758.5453123;
        FloatMath::fract(FloatMath::sin(ALPHA.dot(f32x3::from_array([self, y, z]))) * BETA)
    }
}

impl Random3 for f64 {
    #[inline]
    fn generate(self, y: Self, z: Self) -> Self {
        const ALPHA: f64x3 = f64x3::from_array([12.9898, 78.233, 45.6114]);
        const BETA: f64 = 43758.5453123;
        FloatMath::fract(FloatMath::sin(ALPHA.dot(f64x3::from_array([self, y, z]))) * BETA)
    }
}

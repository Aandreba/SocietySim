use crate::{
    person::PersonStats,
    simd::{f32x2, f32x4},
};

#[cfg_attr(
    not(target_arch = "spirv"),
    derive(Debug, serde::Serialize, serde::Deserialize),
    serde(default)
)]
#[derive(Clone, Copy, PartialEq, Default)]
#[repr(C)]
pub struct Chance {
    pub base: f32,
    pub weight: PersonStats<f32>,
    pub offset: PersonStats<f32>,
}

impl Chance {
    #[inline]
    pub fn apply(self, stats: PersonStats<u8>) -> f32 {
        let (stats_hi, stats_lo) = (
            f32x4::from_array([
                stats.cordiality as f32,
                stats.intelligence as f32,
                stats.knowledge as f32,
                stats.finesse as f32,
            ]),
            f32x2::from_array([stats.gullability as f32, stats.health as f32]),
        );

        let (weight_hi, weight_lo) = self.weight.to_simd();
        let (offset_hi, offset_lo) = self.offset.to_simd();

        let hi = (weight_hi * stats_hi) + offset_hi;
        let lo = (weight_lo * stats_lo) + offset_lo;
        return self.base + hi.reduce_sum() + lo.reduce_sum();
    }
}

use crate::{
    simd::{f32x2, f32x4},
    time::GameDuration,
    ExternBool,
};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(
    not(target_arch = "spirv"),
    derive(Debug, serde::Serialize, serde::Deserialize),
    serde(default)
)]
#[repr(C)]
pub struct PersonStats<T> {
    pub cordiality: T,
    pub intelligence: T,
    pub knowledge: T,
    pub finesse: T,
    pub gullability: T,
    pub health: T,
}

impl PersonStats<f32> {
    #[inline]
    pub fn to_simd(&self) -> (f32x4, f32x2) {
        return (
            f32x4::from_array([
                self.cordiality,
                self.intelligence,
                self.knowledge,
                self.finesse,
            ]),
            f32x2::from_array([self.gullability, self.health]),
        );
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
#[repr(C)]
pub struct Person {
    pub is_male: ExternBool,
    pub age: GameDuration,
    pub stats: PersonStats<u8>,
}
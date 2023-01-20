use crate::{ExternBool, time::GameDuration, simd::{f32x4, f32x2}};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug, serde::Serialize, serde::Deserialize))]
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
    pub fn to_simd (&self) -> (f32x4, f32x2) {
        return (
            f32x4::from_array([self.cordiality, self.intelligence, self.knowledge, self.finesse]),
            f32x2::from_array([self.gullability, self.health])
        )
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

impl Person {
    // #[inline]
    // pub fn affected_stats (&self, _traits: &[Trait]) -> PersonStats<u8> {
    //     todo!()
    // } 
}

#[derive(Clone, Copy, PartialEq, Default)]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
#[repr(C)]
pub struct Trait {
    pub weight: PersonStats<f32>,
    pub offset: PersonStats<i8>
}
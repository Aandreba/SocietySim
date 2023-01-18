use core::simd::{SimdElement, Simd};
use crate::ExternBool;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug, serde::Serialize, serde::Deserialize))]
pub struct PersonStats<T> {
    pub cordiality: T,
    pub intelligence: T,
    pub knowledge: T,
    pub finesse: T,
    pub gullability: T,
    pub health: T
}

impl<T> PersonStats<T> {
    #[inline]
    pub fn as_simd (self) -> (Simd<T, 4>, Simd<T, 2>) where T: SimdElement {
        return (
            Simd::from_array([self.cordiality, self.intelligence, self.knowledge, self.finesse]),
            Simd::from_array([self.gullability, self.health])
        )
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
#[non_exhaustive]
pub struct Person {
    pub is_male: ExternBool,
    pub age: u16, // in weeks
    pub stats: PersonStats<u8>
}
use crate::person::PersonStats;

#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
#[derive(Clone, Copy, PartialEq)]
pub struct Chance {
    pub weight: PersonStats<f32>,
    pub offset: PersonStats<i8>
}

impl Chance {
    #[inline]
    pub fn apply (self, target: PersonStats<u8>) {
        
    }
}
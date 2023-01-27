use serde::{Serialize, Deserialize};
use super::{NamedEntry};

pub type NamedGood<'a> = NamedEntry<'a, Good>;

#[derive(Debug)]
#[repr(C)]
pub struct Good {
    pub base_cost: f32
}

impl Good {
    #[inline]
    pub fn from_raw (raw: RawGood) -> Self {
        return Self {
            base_cost: raw.base_cost
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RawGood {
    pub base_cost: f32
}
use serde::{Serialize, Deserialize};
use super::{NamedEntry};

pub type NamedGood<'a> = NamedEntry<'a, Good>;

#[derive(Debug)]
#[repr(C)]
pub struct Good {

}

impl Good {
    #[inline]
    pub fn from_raw (_raw: RawGood) -> Self {
        return Self {}
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RawGood {

}
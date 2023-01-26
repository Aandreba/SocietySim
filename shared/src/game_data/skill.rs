use serde::{Serialize, Deserialize};
use crate::time::GameDuration;
use super::{Str, NamedEntry};

pub type NamedSkill<'a> = NamedEntry<'a, Skill>;

#[derive(Debug)]
#[repr(C)]
pub struct Skill {
    /// Time that takes to learn the skill
    pub time: GameDuration
}

impl Skill {
    #[inline]
    pub fn from_raw (raw: RawSkill) -> Self {
        return Self {
            time: raw.time
        };
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RawSkill {
    /// Time that takes to learn the skill
    pub time: GameDuration
}
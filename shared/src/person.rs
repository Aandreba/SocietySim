use core::fmt::Debug;
use crate::ExternBool;

#[derive(Clone, PartialEq, Eq, Hash)]
#[repr(C)]
#[non_exhaustive]
pub struct Person {
    pub is_male: ExternBool,
    pub age: u16, // in weeks
    cordiality_intelligence: u8,
    knowledge_finesse: u8,
    gullability_health: u8,
}

impl Person {
    #[inline]
    pub const fn new (is_male: bool, age: u16, cordiality: u8, intelligence: u8, knowledge: u8, finesse: u8, gullability: u8, health: u8) -> Self {
        return Self {
            is_male: ExternBool::new(is_male),
            age,
            cordiality_intelligence: ((cordiality & 0xf) << 4) | (intelligence & 0xf),
            knowledge_finesse: ((knowledge & 0xf) << 4) | (finesse & 0xf),
            gullability_health: ((gullability & 0xf) << 4) | (health & 0xf),
        };
    }

    #[inline]
    pub const fn is_male (&self) -> bool {
        return self.is_male.get()
    }

    #[inline]
    pub const fn cordiality (&self) -> u8 {
        return self.cordiality_intelligence >> 4
    }

    #[inline]
    pub const fn intelligence (&self) -> u8 {
        return self.cordiality_intelligence & 0xf
    }

    #[inline]
    pub fn set_cordiality (&mut self, v: u8) {
        self.cordiality_intelligence = (self.cordiality_intelligence & 0x0f) | ((v & 0xf) << 4)
    }

    #[inline]
    pub fn set_intelligence (&mut self, v: u8) {
        self.cordiality_intelligence = (self.cordiality_intelligence & 0xf0) | (v & 0xf)
    }
}

impl Debug for Person {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Person")
            .field("is_male", &self.is_male())
            .field("age", &self.age)
            .field("cordiality", &self.cordiality())
            .field("intelligence", &self.intelligence())
            .field("knowledge_finesse", &self.knowledge_finesse)
            .field("gullability_health", &self.gullability_health)
            .finish()
    }
}
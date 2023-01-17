use crate::ExternBool;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct Person {
    pub is_male: ExternBool,
    pub age: u16, // in weeks
    pub cordiality_intelligence: u8,
    pub knowledge_finesse: u8,
    pub gullability_health: u8,
}

impl Person {
    #[inline]
    pub fn new (is_male: bool, age: u16, cordiality: u8, intelligence: u8, knowledge: u8, finesse: u8, gullability: u8, health: u8) -> Self {
        return Self {
            is_male: todo!(),
            age,
            cordiality_intelligence: todo!(),
            knowledge_finesse: todo!(),
            gullability_health: todo!(),
        };
    }
}
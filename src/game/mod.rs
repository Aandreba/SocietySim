#[repr(C)]
#[non_exhaustive]
pub struct Person {
    pub is_male: bool,
    pub age: u16, // in weeks
    pub cordiality_intelligence: u8,
    pub knowledge_finesse: u8,
    pub gullability_health: u8,
}
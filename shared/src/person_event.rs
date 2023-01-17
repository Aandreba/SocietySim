use crate::time::GameDuration;

#[cfg_attr(not(target_arch = "spirv"), derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PersonEvent {
    pub duration: Option<GameDuration>,
    pub cordiality: i8,
    pub intelligence: i8,
    pub knowledge: i8,
    pub finesse: i8,
    pub gullability: i8,
    pub health: i8
}
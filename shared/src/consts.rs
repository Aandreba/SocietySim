use crate::population::GenerationOps;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct GeneratePeopleConsts {
    pub ops: GenerationOps,
    pub seed: [u32; 4],
    pub offset: u32
}
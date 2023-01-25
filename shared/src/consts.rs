#[repr(C)]
#[derive(Clone, Copy)]
pub struct GeneratePeopleConsts {
    pub seed: [u32; 4],
    pub offset: u32
}
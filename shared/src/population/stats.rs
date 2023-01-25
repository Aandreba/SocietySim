use crate::person::PersonStats;

#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
#[derive(Default, Clone, Copy)]
#[repr(C)]
pub struct PopulationStats {
    pub males: u32,
    pub stats: PersonStats<u32>
}

#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
#[derive(Default, Clone, Copy)]
#[repr(C)]
pub struct PopulationAdvancedStats {
    pub males: u32,
    
}
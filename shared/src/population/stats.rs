use crate::person::PersonStats;

const LIMIT: usize = u8::MAX as usize + 1;
pub const MAX_AGE: usize = 100;

#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
#[derive(Default, Clone, Copy)]
#[repr(C)]
pub struct PopulationMeanStats {
    pub males: u64,
    pub stats: PersonStats<u64>,
}

#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
#[derive(Clone, Copy)]
#[repr(C)]
pub struct PopulationCountStats {
    pub males: u64,
    pub ages: [u64; MAX_AGE + 1],
    pub stats: PersonStats<[u64; LIMIT]>,
}

impl Default for PopulationCountStats {
    #[inline]
    fn default() -> Self {
        Self {
            males: Default::default(),
            ages: [0; MAX_AGE + 1],
            stats: PersonStats {
                cordiality: [0; LIMIT],
                intelligence: [0; LIMIT],
                knowledge: [0; LIMIT],
                finesse: [0; LIMIT],
                gullability: [0; LIMIT],
                health: [0; LIMIT],
            },
        }
    }
}

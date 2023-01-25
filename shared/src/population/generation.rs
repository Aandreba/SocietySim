use crate::person::PersonStats;

#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
#[derive(Clone, Copy, PartialEq)]
#[repr(C)]
pub struct GenerationOps {
    pub male_chance: f32,
    pub age_mean: u8,
    pub age_std: f32,
    pub stats_mean: PersonStats<u8>,
    pub stats_std: PersonStats<f32>,
}

impl Default for GenerationOps {
    fn default() -> Self {
        Self {
            male_chance: 0.5,
            age_mean: 30,
            age_std: 70.0 / 3.0,
            stats_mean: PersonStats {
                cordiality: 127,
                intelligence: 127,
                knowledge: 127,
                finesse: 127,
                gullability: 127,
                health: 127,
            },
            stats_std: PersonStats {
                cordiality: 42.0,
                intelligence: 42.0,
                knowledge: 42.0,
                finesse: 42.0,
                gullability: 42.0,
                health: 42.0,
            },
        }
    }
}

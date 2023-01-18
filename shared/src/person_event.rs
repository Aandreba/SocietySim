use crate::{
    person::{Person, PersonStats},
    simd::{f32x4, f32x2},
    time::GameDuration,
};

#[cfg_attr(
    not(target_arch = "spirv"),
    derive(Debug, serde::Serialize, serde::Deserialize)
)]
#[derive(Clone, Copy, PartialEq)]
#[repr(C)]
pub struct PersonalEvent {
    pub duration: Option<GameDuration>,
    pub chance: PersonStats<f32>,
    pub effects: PersonStats<i8>,
}

impl PersonalEvent {
    #[inline]
    pub fn calculate_chance(self, person: Person) -> f32 {
        const WEIGHT: f32 = u8::MAX as f32;

        let chance_hi = f32x4::from_array([
            self.chance.cordiality,
            self.chance.finesse,
            self.chance.gullability,
            self.chance.health,
        ]);
        let person_hi = f32x4::from_array([
            person.stats.cordiality as f32,
            person.stats.finesse as f32,
            person.stats.gullability as f32,
            person.stats.health as f32,
        ]);
        let hi = chance_hi * person_hi;

        let chance_lo = f32x2::from_array([
            self.chance.intelligence,
            self.chance.knowledge
        ]);
        let person_lo = f32x2::from_array([
            person.stats.knowledge as f32,
            person.stats.intelligence as f32,
        ]);
        let lo = chance_lo * person_lo;

        return (hi.reduce_sum() + lo.reduce_sum()) / WEIGHT;
    }
}

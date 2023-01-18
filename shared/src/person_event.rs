use crate::{time::GameDuration, person::{Person, PersonStats}, simd::f32x4};

#[cfg_attr(not(target_arch = "spirv"), derive(Debug, serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, PartialEq)]
pub struct PersonalEvent {
    pub duration: Option<GameDuration>,
    pub base_chance: f32,
    pub chance: PersonStats<f32>,
    pub effects: PersonStats<i8>
}

impl PersonalEvent {
    #[inline]
    pub fn calculate_chance (self, person: Person) -> f32 {
        const WEIGHT: f32 = u8::MAX as f32;

        //let chance_hi = f32x4::from_array([self.chance.cordiality, self.chance.finesse, self.chance.gullability, self.chance.health]);
        

        let chance = self.chance.cordiality * person.stats.cordiality as f32
        + self.chance.finesse * person.stats.finesse as f32
        + self.chance.gullability * person.stats.gullability as f32
        + self.chance.health * person.stats.health as f32
        + self.chance.intelligence * person.stats.intelligence as f32
        + self.chance.knowledge * person.stats.knowledge as f32;

        return self.base_chance * chance / WEIGHT
    }
}
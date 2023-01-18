#![cfg_attr(target_arch = "spirv", no_std, feature(asm_experimental_arch))]
// HACK(eddyb) can't easily see warnings otherwise from `spirv-builder` builds.
#![deny(warnings)]

use shared::{person::{Person, PersonStats}, person_event::PersonalEvent};
use spirv_std::{spirv, glam::UVec3};

const CUSTOM_EVENT: PersonalEvent = PersonalEvent {
    duration: None,
    base_chance: 0.75f32,

    chance: PersonStats {
        cordiality: 2f32,
        intelligence: 1f32,
        knowledge: 1f32,
        finesse: 1f32,
        gullability: 1f32,
        health: 1f32,
    },

    effects: PersonStats {
        cordiality: 1,
        intelligence: 0,
        knowledge: 0,
        finesse: 0,
        gullability: 0,
        health: -1,
    },
};

#[spirv(compute(threads(1, 1, 1)))]
pub fn main_cs(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] people: &mut [Person],
) {
    let person = &mut people[id.x as usize];
    let chance = CUSTOM_EVENT.calculate_chance(*person);
    if 0.5f32 < chance {
        person.stats.health = person.stats.health + (CUSTOM_EVENT.effects.health as u8);
    } else {
        person.stats.health = chance as u8;
    }
}
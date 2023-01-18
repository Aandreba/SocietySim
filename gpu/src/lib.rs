#![cfg_attr(target_arch = "spirv", no_std, feature(asm_experimental_arch, asm_const))]
#![feature(bigint_helper_methods)]
// HACK(eddyb) can't easily see warnings otherwise from `spirv-builder` builds.
#![deny(warnings)]

pub mod rand;
pub mod math;

use shared::{person::{Person}, person_event::PersonalEvent, ExternBool};
use spirv_std::{spirv, glam::{UVec3}, macros::debug_printfln};

use crate::rand::Random3;

// Regular odds (1f32 chance) will result in true once every 100 ticks (approximately, obviously) 
//const BASE_CHANCE: f32 = 1f32 / 100f32;
const BASE_CHANCE: f32 = 1f32;

// #[spirv(compute(threads(1)))]
// pub fn main_cs(
//     #[spirv(global_invocation_id)] id: UVec3,
//     #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] people: &mut [Person],
// ) {
//     let person = &mut people[id.x as usize];
//     person.age = GameDuration::default();
// }

// x = # of people, y = # of events
#[spirv(compute(threads(1, 1)))]
pub fn compute_personal_event(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(push_constant)] _seed: &f32,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] people: &[Person],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] events: &[PersonalEvent],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 2)] results: &mut [ExternBool], // [_; x * y]
) {
    let seed = &36f32;
    let person = &people[id.x as usize];
    let event = &events[id.y as usize];
    let chance = BASE_CHANCE * event.calculate_chance(*person);
    unsafe { debug_printfln!("%f", chance) }

    if Random3::generate(id.x as f32, id.y as f32, *seed) < chance {
        let idx = (id.x as usize) * events.len() + (id.y as usize);
        results[idx].set()
    }
}
#![cfg_attr(
    target_arch = "spirv",
    no_std,
    feature(asm_experimental_arch, asm_const)
)]
#![feature(bigint_helper_methods)]
// HACK(eddyb) can't easily see warnings otherwise from `spirv-builder` builds.
#![deny(warnings)]

pub mod math;
pub mod rand;

use rand::Random;
use shared::{person::{Person, PersonStats}, person_event::PersonalEvent, time::GameDuration, ExternBool, simd::f32x2};
use spirv_std::{glam::UVec3, macros::debug_printfln, spirv};

// Regular odds (1f32 chance) will result in true once every 100 ticks (approximately, obviously)
//const BASE_CHANCE: f32 = 1f32 / 100f32;
const BASE_CHANCE: f32 = 1f32;

// x = # of people
#[spirv(compute(threads(1)))]
pub fn generate_people(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(push_constant)] random: &Random,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] people: &mut [Person],
) {
    let mut random = *random;
    random.apply_entropy(f32x2::from_array([id.x as f32, id.x as f32]));

    people[id.x as usize] = Person {
        is_male: ExternBool::new(random.next_bool()),
        age: GameDuration::from_days(random.next_u16() % 36500),
        stats: PersonStats {
            cordiality: random.next_u8(),
            intelligence: random.next_u8(),
            knowledge: random.next_u8(),
            finesse: random.next_u8(),
            gullability: random.next_u8(),
            health: random.next_u8(),
        },
    }
}

// x = # of people, y = # of events
#[spirv(compute(threads(1, 1)))]
pub fn compute_personal_event(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(push_constant)] random: &Random,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] people: &[Person],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] events: &[PersonalEvent],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 2)] results: &mut [ExternBool], // [_; x * y]
) {
    let mut random = *random;
    random.apply_entropy(f32x2::from_array([id.x as f32, id.y as f32]));

    let person = &people[id.x as usize];
    let event = &events[id.y as usize];
    let chance = BASE_CHANCE * event.calculate_chance(*person);
    unsafe { debug_printfln!("%f", chance) }

    if random.next_f32() < chance {
        let idx = (id.x as usize) * events.len() + (id.y as usize);
        results[idx].set()
    }
}

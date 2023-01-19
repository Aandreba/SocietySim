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

use crate::rand::{Random2, Random3};
use shared::{person::{Person, PersonStats}, person_event::PersonalEvent, time::GameDuration, ExternBool};
use spirv_std::{glam::UVec3, macros::debug_printfln, spirv};

// Regular odds (1f32 chance) will result in true once every 100 ticks (approximately, obviously)
//const BASE_CHANCE: f32 = 1f32 / 100f32;
const BASE_CHANCE: f32 = 1f32;

// x = # of people
#[spirv(compute(threads(1, 1)))]
pub fn generate_people(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(push_constant)] seed: &f32,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] people: &mut [Person],
) {
    #[inline]
    fn random(seed: &mut f32, x: f32) -> f32 {
        let v = Random2::generate(*seed, x);
        *seed = 100f32 * v;
        return v;
    }

    #[cfg(target_feature = "Float64")]
    #[inline]
    fn random_f64(seed: &mut f32, x: f32) -> f64 {
        let v = Random2::generate(*seed as f64, x as f64);
        *seed = (100f64 * v) as f32;
        return v;
    }

    let mut seed = *seed;
    let x = id.x as f32;

    people[id.x as usize] = Person {
        is_male: ExternBool::new(random(&mut seed, x) >= 0.5f32),
        #[cfg(target_feature = "Float64")]
        age: GameDuration::from_days((random_f64(&mut seed, x) * 36500f64) as u16),
        #[cfg(not(target_feature = "Float64"))]
        age: GameDuration::from_days((random(&mut seed, x) * 36500f32) as u16),
        stats: PersonStats {
            cordiality: (255f32 * random(&mut seed, x)) as u8,
            intelligence: (255f32 * random(&mut seed, x)) as u8,
            knowledge: (255f32 * random(&mut seed, x)) as u8,
            finesse: (255f32 * random(&mut seed, x)) as u8,
            gullability: (255f32 * random(&mut seed, x)) as u8,
            health: (255f32 * random(&mut seed, x)) as u8,
        },
    }
}

// x = # of people, y = # of events
#[spirv(compute(threads(1, 1)))]
pub fn compute_personal_event(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(push_constant)] seed: &f32,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] people: &[Person],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] events: &[PersonalEvent],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 2)] results: &mut [ExternBool], // [_; x * y]
) {
    let person = &people[id.x as usize];
    let event = &events[id.y as usize];
    let chance = BASE_CHANCE * event.calculate_chance(*person);
    unsafe { debug_printfln!("%f", chance) }

    if Random3::generate(id.x as f32, id.y as f32, *seed) < chance {
        let idx = (id.x as usize) * events.len() + (id.y as usize);
        results[idx].set()
    }
}

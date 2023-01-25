#![cfg_attr(
    target_arch = "spirv",
    no_std,
    feature(asm_experimental_arch, asm_const)
)]
#![feature(bigint_helper_methods, concat_idents)]
// HACK(eddyb) can't easily see warnings otherwise from `spirv-builder` builds.
#![deny(warnings)]

pub mod math;
pub mod rand;

use rand::Random;
use shared::{
    consts::GeneratePeopleConsts,
    person::{Person, PersonStats, stats::PopulationStats},
    time::GameDuration,
    ExternBool,
};
use spirv_std::{glam::UVec3, spirv, arch::atomic_i_add};

// x = # of people
#[spirv(compute(threads(1)))]
pub fn generate_people(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(push_constant)] consts: &GeneratePeopleConsts,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] people: &mut [Person],
) {
    let id = id.x + consts.offset;
    let mut random = Random::from_entropy(consts.seed, id);
    random.jump();

    unsafe {
        core::ptr::write(
            &mut people[id as usize],
            Person {
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
            },
        );
    }
}

#[spirv(compute(threads(1)))]
pub fn population_stats(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] people: &[Person],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] stats: &mut PopulationStats,
) {
    let person = &people[id.x as usize];
    if person.is_male.get() {
        unsafe { atomic_i_add::<_, 2, 0>(&mut stats.males, 1) };
    }

    unsafe {
        let _ = atomic_i_add::<_, 2, 0>(&mut stats.stats.cordiality, person.stats.cordiality as u32);
        let _ = atomic_i_add::<_, 2, 0>(&mut stats.stats.finesse, person.stats.finesse as u32);
        let _ = atomic_i_add::<_, 2, 0>(&mut stats.stats.gullability, person.stats.gullability as u32);
        let _ = atomic_i_add::<_, 2, 0>(&mut stats.stats.health, person.stats.health as u32);
        let _ = atomic_i_add::<_, 2, 0>(&mut stats.stats.intelligence, person.stats.intelligence as u32);
        let _ = atomic_i_add::<_, 2, 0>(&mut stats.stats.knowledge, person.stats.knowledge as u32);
    }  
}

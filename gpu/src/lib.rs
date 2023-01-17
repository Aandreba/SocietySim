#![cfg_attr(target_arch = "spirv", no_std)]
// HACK(eddyb) can't easily see warnings otherwise from `spirv-builder` builds.
#![deny(warnings)]

use shared::person::Person;
use spirv_std::{spirv, glam::UVec3};

#[spirv(compute(threads(5, 1, 1)))]
pub fn main_cs(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] people: &mut [Person],
) {
    people[id.x as usize].set_cordiality(5);
}

pub fn caluclate_reproduction () {
    
}
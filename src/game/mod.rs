#[repr(u8)]
pub enum Gender {
    Male = 0,
    Female = 1
}

#[repr(C)]
pub struct Person {
    gender: Gender,
    charism: u8,
}
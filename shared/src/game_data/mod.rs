use core::hash::Hash;

pub type Str = Box<str>;

pub mod job;
pub mod skill;
pub mod building;
pub mod good;
pub mod institution;

#[derive(Debug, Clone, Copy)]
pub struct NamedEntry<'a, T: ?Sized> {
    pub key: &'a Str,
    pub value: &'a T
}

impl<T: ?Sized> PartialEq for NamedEntry<'_, T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl<T: ?Sized> Hash for NamedEntry<'_, T> {
    #[inline]
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.key.hash(state);
    }
}

impl<'a, T: ?Sized> From<(&'a Str, &'a T)> for NamedEntry<'a, T> {
    #[inline]
    fn from((key, value): (&'a Str, &'a T)) -> Self {
        return Self { key, value }
    }
}

impl<T: ?Sized> Eq for NamedEntry<'_, T> {}
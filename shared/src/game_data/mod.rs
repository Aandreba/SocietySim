use core::hash::Hash;
use vector_mapp::r#box::BoxMap;

pub type Str = Box<str>;

pub mod job;
pub mod skill;
pub mod building;
pub mod good;

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

impl<'a, T: ?Sized> Into<(&'a Str, &'a T)> for NamedEntry<'a, T> {
    #[inline]
    fn into(self) -> (&'a Str, &'a T) {
        (self.key, self.value)
    }
}

impl<T: ?Sized> Eq for NamedEntry<'_, T> {}

#[inline]
pub(super) fn try_get_key_value<'a, T> (map: &'a BoxMap<Str, T>, name: &str) -> anyhow::Result<NamedEntry<'a, T>> {
    match map.get_key_value(name) {
        Some(x) => Ok(x.into()),
        None => Err(anyhow::Error::msg(format!("entry with key '{name}' not found on map of '{}'", core::any::type_name::<T>())))
    }
}
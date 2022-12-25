use std::num::NonZeroU64;

#[derive(Debug)]
pub struct Queue {
    pub(crate) inner: NonZeroU64
}
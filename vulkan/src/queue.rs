use std::{num::NonZeroU64};

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Queue {
    pub(crate) inner: NonZeroU64
}
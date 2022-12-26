use std::{num::NonZeroU64};

#[derive(Debug, Clone, Copy)]
pub struct Queue {
    pub(crate) inner: NonZeroU64
}
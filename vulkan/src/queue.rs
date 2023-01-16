use std::{num::NonZeroU64};
use crate::device::Device;

#[derive(Debug, Clone, Copy, PartialEq, Hash)]
pub struct Queue<'a> {
    pub(super) inner: NonZeroU64,
    pub(super) index: u32,
    pub(super) parent: &'a Device
}
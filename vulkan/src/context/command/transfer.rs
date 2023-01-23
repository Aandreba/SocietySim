use super::Command;
use crate::{buffer::Buffer, Entry, Result, utils::usize_to_u32, alloc::DeviceAllocator, sync::Fence, context::Context};
use std::{
    marker::PhantomData,
    ops::{Bound, RangeBounds, Deref},
};

pub struct TransferCommand<'a, 'b> {
    cmd: Command<'a>,
    _phtm: PhantomData<&'b mut &'b ()>,
}

impl<'a, 'b> TransferCommand<'a, 'b> {
    #[inline]
    pub(crate) fn new (cmd: Command<'a>) -> Self {
        return Self { cmd, _phtm: PhantomData }
    }

    #[inline]
    pub fn buffer_copy<T: Copy, S: DeviceAllocator, D: DeviceAllocator>(
        self,
        src: &'b Buffer<T, S>,
        dst: &'b mut Buffer<T, D>,
        regions: impl IntoIterator<Item = BufferCopyRegion<impl RangeBounds<vk::DeviceSize>, T>>,
    ) -> Self {
        let regions = regions.into_iter().map(|x| x.into_buffer_copy(dst)).collect::<Vec<_>>();
        (Entry::get().cmd_copy_buffer)(
            self.cmd.buffer(),
            src.id(),
            dst.id(),
            usize_to_u32(regions.len()),
            regions.as_ptr()
        );
        self
    }

    #[inline]
    pub fn execute (self) -> Result<()> {
        let fence = self.cmd.submit()?;
        todo!()
    }
}

pub struct TransferEvent<'a, 'b> {
    fence: Fence<&'a Context>,
    _phtm: PhantomData<&'b mut &'b ()>
}

/// Two-buffer region where a copy is to be done by Vulkan
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BufferCopyRegion<B, T> {
    pub src_offset: Bound<vk::DeviceSize>,
    pub dst_bounds: B,
    _phtm: PhantomData<T>,
}

impl<T, B: RangeBounds<vk::DeviceSize>> BufferCopyRegion<B, T> {
    #[inline]
    pub fn new(src_offset: Option<Bound<vk::DeviceSize>>, dst_bounds: B) -> Self {
        return Self {
            src_offset: src_offset.unwrap_or_else(|| dst_bounds.start_bound().cloned()),
            dst_bounds,
            _phtm: PhantomData,
        };
    }
}

impl<T, B: RangeBounds<vk::DeviceSize>> BufferCopyRegion<B, T> {
    const ELEMENT_SIZE: vk::DeviceSize = core::mem::size_of::<T>() as vk::DeviceSize;

    #[inline]
    pub fn into_buffer_copy<A: DeviceAllocator> (self, dst: &Buffer<T, A>) -> vk::BufferCopy {
        let src_offset = match self.src_offset {
            Bound::Excluded(x) => Self::ELEMENT_SIZE * (x + 1),
            Bound::Included(x) => Self::ELEMENT_SIZE * x,
            Bound::Unbounded => 0,
        };

        let dst_offset = match self.dst_bounds.start_bound() {
            Bound::Included(x) => Self::ELEMENT_SIZE * (x + 1),
            Bound::Excluded(x) => Self::ELEMENT_SIZE * x,
            Bound::Unbounded => 0,
        };

        let dst_end = match self.dst_bounds.end_bound() {
            Bound::Included(x) => Self::ELEMENT_SIZE * (x + 1),
            Bound::Excluded(x) => Self::ELEMENT_SIZE * x,
            Bound::Unbounded => dst.size(),
        };

        return vk::BufferCopy {
            srcOffset: src_offset,
            dstOffset: dst_offset,
            size: dst_end - dst_offset,
        };
    }
}

impl<T, B: RangeBounds<vk::DeviceSize>> From<B> for BufferCopyRegion<B, T> {
    #[inline]
    fn from(dst_bounds: B) -> Self {
        Self::new(None, dst_bounds)
    }
}
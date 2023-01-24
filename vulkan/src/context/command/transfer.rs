use super::Command;
use crate::{
    alloc::DeviceAllocator,
    buffer::Buffer,
    context::{event::Event, ContextRef},
    forward_phantom,
    utils::usize_to_u32,
    Entry, Result,
};
use std::{
    marker::PhantomData,
    mem::MaybeUninit,
    ops::{Bound, RangeBounds}, pin::Pin,
};

#[derive(Debug)]
pub struct TransferCommand<'b, C: ContextRef> {
    cmd: Command<C>,
    _phtm: PhantomData<&'b mut &'b ()>,
}

impl<'b, C: ContextRef> TransferCommand<'b, C> {
    #[inline]
    pub(crate) fn new(cmd: Command<C>) -> Self {
        return Self {
            cmd,
            _phtm: PhantomData,
        };
    }

    #[inline]
    pub fn buffer_write<T: Copy, A: DeviceAllocator>(
        self,
        src: &'b [T],
        dst: &'b mut Buffer<T, A>,
        dst_offset: vk::DeviceSize,
    ) -> ::std::result::Result<Self, Self> {
        const DATA_SIZE_MAX: vk::DeviceSize = 65536;

        let offset = dst_offset * Buffer::<T, A>::BYTES_PER_ELEMENT;
        if offset % 4 != 0 {
            #[cfg(debug_assertions)]
            eprintln!(
                "offset ({dst_offset} * {0}) must be a multiple of 4: {offset} % 4 != 0",
                Buffer::<T, A>::BYTES_PER_ELEMENT
            );
            return Err(self);
        }

        let size = (src.len() as u64) * Buffer::<T, A>::BYTES_PER_ELEMENT;
        if DATA_SIZE_MAX > 65536 || size % 4 != 0 {
            #[cfg(debug_assertions)]
            eprintln!("size ({0} * {1}) must be  less than or equal {2} to and a multiple of 4, so either [{offset} % 4 != 0] or [{offset} > {2}]", src.len(), Buffer::<T, A>::BYTES_PER_ELEMENT, DATA_SIZE_MAX);
            return Err(self);
        }

        (Entry::get().cmd_update_buffer)(
            self.cmd.buffer(),
            dst.id(),
            dst_offset * Buffer::<T, A>::BYTES_PER_ELEMENT,
            (src.len() as u64) * Buffer::<T, A>::BYTES_PER_ELEMENT,
            src.as_ptr().cast(),
        );

        return Ok(self);
    }

    #[inline]
    pub fn buffer_copy<T: Copy, S: DeviceAllocator, D: DeviceAllocator>(
        self,
        src: &'b Buffer<T, S>,
        dst: &'b mut Buffer<T, D>,
        regions: impl IntoIterator<Item = BufferCopyRegion<impl RangeBounds<vk::DeviceSize>, T>>,
    ) -> Self {
        let regions = regions
            .into_iter()
            .map(|x| x.into_buffer_copy(dst))
            .collect::<Vec<_>>();
        (Entry::get().cmd_copy_buffer)(
            self.cmd.buffer(),
            src.id(),
            dst.id(),
            usize_to_u32(regions.len()),
            regions.as_ptr(),
        );
        self
    }

    #[inline]
    pub fn execute(self) -> Result<Event<Pin<C>, TransferConsumer<'b>>> {
        let fence = self.cmd.submit()?;
        return Ok(Event::new(fence, TransferConsumer::new()));
    }
}

impl<'b, C: ContextRef> TransferCommand<'b, C> {
    #[inline]
    pub fn buffer_write_init<T: Copy, A: DeviceAllocator>(
        self,
        src: &'b [T],
        dst: &'b mut Buffer<MaybeUninit<T>, A>,
        dst_offset: vk::DeviceSize,
    ) -> ::std::result::Result<Self, Self> {
        self.buffer_write(
            unsafe { &*(src as *const [T] as *const [MaybeUninit<T>]) },
            dst,
            dst_offset,
        )
    }

    #[inline]
    pub fn buffer_copy_init<T: Copy, S: DeviceAllocator, D: DeviceAllocator>(
        self,
        src: &'b Buffer<T, S>,
        dst: &'b mut Buffer<MaybeUninit<T>, D>,
        regions: impl IntoIterator<Item = BufferCopyRegion<impl RangeBounds<vk::DeviceSize>, T>>,
    ) -> Self {
        return self.buffer_copy(
            unsafe { &*(src as *const Buffer<T, S>).cast::<Buffer<MaybeUninit<T>, S>>() },
            dst,
            regions.into_iter().map(BufferCopyRegion::as_uninit),
        );
    }
}

forward_phantom! {
    &'b mut &'b () as pub TransferConsumer<'b,>
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

    #[inline]
    pub fn as_uninit(self) -> BufferCopyRegion<B, MaybeUninit<T>> {
        return BufferCopyRegion {
            src_offset: self.src_offset,
            dst_bounds: self.dst_bounds,
            _phtm: PhantomData,
        };
    }
}

impl<T, B: RangeBounds<vk::DeviceSize>> BufferCopyRegion<B, T> {
    const ELEMENT_SIZE: vk::DeviceSize = core::mem::size_of::<T>() as vk::DeviceSize;

    #[inline]
    pub fn into_buffer_copy<A: DeviceAllocator>(self, dst: &Buffer<T, A>) -> vk::BufferCopy {
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

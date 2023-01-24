use super::{Command};
use crate::{
    context::{ContextRef, event::Event, Context},
    descriptor::DescriptorSet,
    pipeline::{Pipeline, PipelineBindPoint},
    shader::ShaderStages,
    utils::usize_to_u32,
    Entry, Result, forward_phantom,
};
use std::{
    ffi::c_void,
    ops::{Bound, Index, RangeBounds},
};

#[derive(Debug)]
pub struct ComputeCommand<'a, 'b, C: ContextRef> {
    cmd: Command<'a>,
    pipeline: &'b Pipeline<C>,
}

impl<'a, 'b, C: ContextRef> ComputeCommand<'a, 'b, C> {
    #[inline]
    pub(crate) fn new<R: RangeBounds<usize>>(
        cmd: Command<'a>,
        pipeline: &'b Pipeline<C>,
        desc_sets: R,
    ) -> Result<Self>
    where
        [DescriptorSet]: Index<R, Output = [DescriptorSet]>,
    {
        (Entry::get().cmd_bind_pipeline)(cmd.buffer(), PipelineBindPoint::Compute as i32, pipeline.id());

        let first_set = match desc_sets.start_bound() {
            Bound::Excluded(x) => usize_to_u32(*x + 1),
            Bound::Included(x) => usize_to_u32(*x),
            Bound::Unbounded => 0,
        };

        let descriptor_set_count = usize_to_u32(match desc_sets.end_bound() {
            Bound::Excluded(x) => *x,
            Bound::Included(x) => *x + 1,
            Bound::Unbounded => pipeline.sets().len(),
        }) - first_set;

        let descriptor_sets: &[DescriptorSet] = &pipeline.sets().deref()[desc_sets];

        (Entry::get().cmd_bind_descriptor_sets)(
            cmd.buffer(),
            PipelineBindPoint::Compute as i32,
            pipeline.layout(),
            first_set,
            descriptor_set_count,
            descriptor_sets.as_ptr().cast(),
            0,
            core::ptr::null(),
        );

        return Ok(Self {
            cmd,
            pipeline
        })
    }

    #[inline]
    pub fn push_contant<T: Copy>(
        self,
        value: &T,
    ) -> Self {
        (Entry::get().cmd_push_constants)(
            self.cmd.buffer(),
            self.pipeline.layout(),
            ShaderStages::COMPUTE.bits(),
            0,
            usize_to_u32(core::mem::size_of::<T>()),
            value as *const T as *const c_void,
        );
        return self;
    }

    #[inline]
    pub fn dispatch (self, x: u32, y: u32, z: u32) -> Result<Event<&'a Context, ComputeConsumer>> {
        (Entry::get().cmd_dispatch)(self.cmd.buffer(), x, y, z);
        let fence = self.cmd.submit()?;
        return Ok(Event::new(fence, ComputeConsumer::new()))
    }
}

forward_phantom! {
    () as pub ComputeConsumer
}
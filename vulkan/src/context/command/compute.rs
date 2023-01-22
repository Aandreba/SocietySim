use super::{Command, CommandResult};
use crate::{
    context::{ContextRef, QueueFamily},
    descriptor::DescriptorSet,
    pipeline::{Pipeline, PipelineBindPoint},
    shader::ShaderStages,
    utils::usize_to_u32,
    Entry, Result,
};
use std::{
    ffi::c_void,
    num::NonZeroU64,
    ops::{Bound, Index, RangeBounds},
    sync::MutexGuard,
};

pub struct ComputeCommand<'a, 'b, C: ContextRef> {
    cmd: Command<'a>,
    pipeline: &'b Pipeline<C>,
}

impl<'a, 'b, C: ContextRef> ComputeCommand<'a, 'b, C> {
    #[inline]
    pub(crate) fn new<R: RangeBounds<usize>>(
        family: &'a QueueFamily,
        pool_buffer: MutexGuard<'a, [NonZeroU64; 2]>,
        point: PipelineBindPoint,
        pipeline: &'b Pipeline<C>,
        desc_sets: R,
    ) -> Result<Self>
    where
        [DescriptorSet]: Index<R, Output = [DescriptorSet]>,
    {
        let cmd = Command::new(family, pool_buffer)?;

        (Entry::get().cmd_bind_pipeline)(cmd.buffer(), point as i32, pipeline.id());

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
            point as i32,
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
        &mut self,
        value: &T,
        stages: ShaderStages,
    ) -> CommandResult<'a, ()> {
        (Entry::get().cmd_push_constants)(
            self.id(),
            self.pipeline.layout(),
            stages.bits(),
            0,
            usize_to_u32(core::mem::size_of::<T>()),
            value as *const T as *const c_void,
        );
        return Ok(());
    }

    #[inline]
    pub fn execute (self, x: u32, y: u32, z: u32) {
        (Entry::get().cmd_dispatch)(self.id(), x, y, z);
        self.cmd.submit();
    }
}

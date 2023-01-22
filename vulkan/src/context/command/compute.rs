use std::{ffi::c_void, ops::{Bound, RangeBounds}, sync::MutexGuard, num::NonZeroU64};
use crate::{Entry, shader::ShaderStages, Result, utils::usize_to_u32, descriptor::DescriptorSet, pool::PipelineBindPoint, pipeline::Pipeline, context::QueueFamily};
use super::{Command, CommandResult};

pub struct ComputeCommand<'a> (Command<'a>);

impl<'a> ComputeCommand<'a> {
    #[inline]
    pub(crate) fn new(family: &'a QueueFamily, pool_buffer: MutexGuard<'a, [NonZeroU64; 2]>) -> Result<Self> {
        Command::new(family, pool_buffer).map(Self)
    }

    #[inline]
    pub fn push_contant<T: Copy> (&mut self, value: &T, stages: ShaderStages) -> CommandResult<'a, ()> {
        let pipeline = self.pipeline.ok_or(vk::ERROR_NOT_PERMITTED_KHR)?;
        (Entry::get().cmd_push_constants)(
            self.id(),
            pipeline.layout(),
            stages.bits(),
            0,
            usize_to_u32(core::mem::size_of::<T>()),
            value as *const T as *const c_void
        );
        return Ok(())
    }

    #[inline]
    pub fn bind_pipeline<R: RangeBounds<usize>> (&mut self, point: PipelineBindPoint, pipeline: &'a Pipeline<P>, desc_sets: R) where [DescriptorSet]: Index<R, Output = [DescriptorSet]> {
        (Entry::get().cmd_bind_pipeline)(
            self.id(),
            point as i32,
            pipeline.id()
        );

        let first_set = match desc_sets.start_bound() {
            Bound::Excluded(x) => usize_to_u32(*x + 1),
            Bound::Included(x) => usize_to_u32(*x),
            Bound::Unbounded => 0
        };

        let descriptor_set_count = usize_to_u32(match desc_sets.end_bound() {
            Bound::Excluded(x) => *x,
            Bound::Included(x) => *x + 1,
            Bound::Unbounded => pipeline.sets().len()
        }) - first_set;

        let descriptor_sets: &[DescriptorSet] = &pipeline.sets().deref()[desc_sets];

        (Entry::get().cmd_bind_descriptor_sets)(
            self.id(),
            point as i32,
            pipeline.layout(),
            first_set,
            descriptor_set_count,
            descriptor_sets.as_ptr().cast(),
            0,
            core::ptr::null()
        );
        self.pipeline = Some(pipeline);
    }

    #[inline]
    pub fn dispatch (self, x: u32, y: u32, z: u32) {
        (Entry::get().cmd_dispatch)(
            self.id(),
            x, y, z
        );
    }
}

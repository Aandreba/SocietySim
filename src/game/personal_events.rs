use rand::{distributions::OpenClosed01, thread_rng, Rng};
use shared::{person::Person, person_event::PersonalEvent, ExternBool};
use vulkan::{
    alloc::{DeviceAllocator, MemoryFlags},
    buffer::{Buffer, UsageFlags, BufferFlags},
    device::DeviceRef,
    pipeline::{ComputeBuilder, Pipeline},
    pool::{CommandBufferUsage, PipelineBindPoint},
    Result, descriptor::{DescriptorSet, DescriptorType}, utils::u64_to_u32, shader::ShaderStages, cstr, sync::{FenceFlags, Fence},
};

use crate::context::Context;

pub struct PersonalEvents<D: DeviceRef> {
    pipeline: Pipeline<D>,
    seed: f32,
}

impl<D: DeviceRef> PersonalEvents<D> {
    #[inline]
    pub fn new (dev: D, words: &[u32]) -> Result<Self> where D: Clone {
        let pipeline = ComputeBuilder::new(dev)
            .entry(cstr!("compute_personal_event"))
            .binding(DescriptorType::StorageBuffer, 1)
            .binding(DescriptorType::StorageBuffer, 1)
            .binding(DescriptorType::StorageBuffer, 1)
            .build(words)?;

        return Ok(Self {
            pipeline,
            seed: 100f32 * thread_rng().sample::<f32, _>(OpenClosed01),
        });
    }

    #[inline]
    pub fn call<Ctx: DeviceRef, P: Clone + DeviceAllocator, E: DeviceAllocator>(
        &mut self,
        people: &Buffer<Person, P>,
        events: &Buffer<PersonalEvent, E>,
        ctx: &mut Context<Ctx>
    ) -> Result<Buffer<ExternBool, P>> {
        let result = Buffer::<ExternBool, _>::new_uninit(
            people.len() * events.len(),
            UsageFlags::STORAGE_BUFFER,
            BufferFlags::empty(),
            MemoryFlags::MAPABLE,
            people.alloc().clone(),
        )?;

        let set: &DescriptorSet = self.pipeline.sets().first().unwrap();
        let people_desc = set.write_descriptor(people, 0);
        let events_desc = set.write_descriptor(events, 0);
        let result_desc = set.write_descriptor(&result, 0);
        self.pipeline.sets_mut().update(&[people_desc, events_desc, result_desc]);

        let mut cmd_buff = ctx.pool.begin_mut(0, CommandBufferUsage::ONE_TIME_SUBMIT)?;
        cmd_buff.bind_pipeline(PipelineBindPoint::Compute, &self.pipeline, ..);
        cmd_buff.push_contant(&self.seed, ShaderStages::COMPUTE)?;
        cmd_buff.dispatch(u64_to_u32(people.len()), u64_to_u32(events.len()), 1);
        drop(cmd_buff);

        let mut fence = Fence::new(self.pipeline.device(), FenceFlags::empty())?;
        fence.bind_to::<_, Ctx>(&mut ctx.pool, &mut ctx.queue, None)?;
        fence.wait(None)?;

        self.seed = 100f32 * thread_rng().sample::<f32, _>(OpenClosed01);
        return unsafe { Ok(result.assume_init()) };
    }
}

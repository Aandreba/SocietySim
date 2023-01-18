use std::mem::MaybeUninit;
use rand::{distributions::OpenClosed01, rngs::ThreadRng, thread_rng, Rng};
use shared::{person::Person, person_event::PersonalEvent, ExternBool};
use vulkan::{
    alloc::{DeviceAllocator, MemoryFlags},
    buffer::{Buffer, UsageFlags, BufferFlags},
    device::DeviceRef,
    pipeline::{ComputeBuilder, Pipeline},
    pool::{CommandPool, CommandBufferUsage, PipelineBindPoint},
    Result, descriptor::{DescriptorSet, DescriptorType}, utils::u64_to_u32, queue::{Queue, FenceFlags, Fence}, shader::ShaderStages,
};
use crate::cstr;

pub struct GeneratePeople<D: DeviceRef> {
    pipeline: Pipeline<D>,
    rng: ThreadRng,
    seed: f32,
}

impl<D: DeviceRef> GeneratePeople<D> {
    #[inline]
    pub fn new (dev: D, words: &[u32]) -> Result<Self> where D: Clone {
        let pipeline = ComputeBuilder::new(dev)
            .entry(cstr!("generate_people"))
            .binding(DescriptorType::StorageBuffer, 1)
            .build(words)?;

        let mut rng = thread_rng();
        return Ok(Self {
            pipeline,
            seed: 100f32 * rng.sample::<f32, _>(OpenClosed01),
            rng,
        });
    }

    #[inline]
    pub fn call<Pool: DeviceRef, P: DeviceAllocator>(
        &mut self,
        people: Buffer<MaybeUninit<Person>, P>,
        pool: &mut CommandPool<Pool>,
        queue: &mut Queue,
    ) -> Buffer<Person, P> {
        let set: &DescriptorSet = self.pipeline.sets().first().unwrap();
        let people_desc = set.write_descriptor(&people, 0);
        self.pipeline.sets_mut().update(&[people_desc]);

        let mut cmd_buff = pool.begin_mut(0, CommandBufferUsage::ONE_TIME_SUBMIT)?;
        cmd_buff.bind_pipeline(PipelineBindPoint::Compute, &self.pipeline, ..);
        cmd_buff.push_contant(&self.seed, ShaderStages::COMPUTE)?;
        cmd_buff.dispatch(u64_to_u32(people.len()), u64_to_u32(events.len()), 1);
        drop(cmd_buff);

        let mut fence = Fence::new(self.pipeline.device(), FenceFlags::empty())?;
        fence.bind_to::<_, Pool>(pool, queue, None)?;
        fence.wait(None)?;

        self.seed = 100f32 * self.rng.sample::<f32, _>(OpenClosed01);
        return unsafe { Ok(people.assume_init()) };
    }
}

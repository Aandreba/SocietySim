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

pub struct PersonalEvents<D: DeviceRef> {
    pipeline: Pipeline<D>,
    rng: ThreadRng,
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

        let mut rng = thread_rng();
        return Ok(Self {
            pipeline,
            seed: 100f32 * rng.sample::<f32, _>(OpenClosed01),
            rng,
        });
    }

    #[inline]
    pub fn call<Pool: DeviceRef, P: Clone + DeviceAllocator, E: DeviceAllocator>(
        &mut self,
        people: &Buffer<Person, P>,
        events: &Buffer<PersonalEvent, E>,
        pool: &mut CommandPool<Pool>,
        queue: &mut Queue,
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

        let mut cmd_buff = pool.begin_mut(0, CommandBufferUsage::ONE_TIME_SUBMIT)?;
        cmd_buff.bind_pipeline(PipelineBindPoint::Compute, &self.pipeline, ..);
        cmd_buff.push_contant(&self.seed, ShaderStages::COMPUTE)?;
        cmd_buff.dispatch(u64_to_u32(people.len()), u64_to_u32(events.len()), 1);
        drop(cmd_buff);

        let mut fence = Fence::new(self.pipeline.device(), FenceFlags::empty())?;
        fence.bind_to::<_, Pool>(pool, queue, None)?;
        fence.wait(None)?;

        self.seed = 100f32 * self.rng.sample::<f32, _>(OpenClosed01);
        return unsafe { Ok(result.assume_init()) };
    }
}

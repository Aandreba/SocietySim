use std::time::Duration;

use rand::{distributions::OpenClosed01, thread_rng, Rng};
use shared::{person::Person, person_event::PersonalEvent, ExternBool};
use vulkan::{
    alloc::{DeviceAllocator, MemoryFlags},
    buffer::{Buffer, UsageFlags, BufferFlags},
    pipeline::{ComputeBuilder, Pipeline},
    Result, descriptor::{DescriptorSet, DescriptorType}, utils::u64_to_u32, shader::ShaderStages, cstr, sync::{FenceFlags, Fence}, include_spv, context::{ContextRef, Context},
};

pub struct PersonalEvents<C: ContextRef> {
    pipeline: Pipeline<C>,
    seed: f32,
}

impl<C: ContextRef> PersonalEvents<C> {
    #[inline]
    pub fn new (context: C) -> Result<Self> where C: Clone {
        const WORDS: &[u32] = include_spv!("compute_personal_event.spv");

        let pipeline = ComputeBuilder::new(context)
            .entry(cstr!("compute_personal_event"))
            .binding(DescriptorType::StorageBuffer, 1)
            .binding(DescriptorType::StorageBuffer, 1)
            .binding(DescriptorType::StorageBuffer, 1)
            .build(WORDS)?;

        return Ok(Self {
            pipeline,
            seed: 100f32 * thread_rng().sample::<f32, _>(OpenClosed01),
        });
    }

    #[inline]
    pub fn context (&self) -> &Context {
        return self.pipeline.context()
    }

    #[inline]
    pub fn call<P: Clone + DeviceAllocator, E: DeviceAllocator>(
        &mut self,
        people: &Buffer<Person, P>,
        events: &Buffer<PersonalEvent, E>
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

        // TODO maybe this is not thread-safe. dsc_sets may need external synchronization
        self.pipeline.compute(..)?
            .push_contant(&self.seed)
            .dispatch(u64_to_u32(people.len()), u64_to_u32(events.len()), 1)?;

        std::thread::sleep(Duration::from_secs(2));
        // let mut fence = Fence::new(self.pipeline.device(), FenceFlags::empty())?;
        // fence.bind_to::<_, Ctx>(&mut ctx.pool, &mut ctx.queue, None)?;
        // fence.wait(None)?;

        self.seed = 100f32 * thread_rng().sample::<f32, _>(OpenClosed01);
        return unsafe { Ok(result.assume_init()) };
    }
}

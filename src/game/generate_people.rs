use std::{mem::MaybeUninit, time::Duration};
use rand::{distributions::OpenClosed01, thread_rng, Rng};
use shared::{person::Person};
use vulkan::{
    alloc::{DeviceAllocator, MemoryFlags},
    buffer::{Buffer, UsageFlags, BufferFlags},
    pipeline::{ComputeBuilder, Pipeline},
    Result, descriptor::{DescriptorSet, DescriptorType}, utils::u64_to_u32, shader::ShaderStages, cstr, sync::{Fence, FenceFlags}, include_spv, context::ContextRef,
};

pub struct GeneratePeople<C: ContextRef> {
    pipeline: Pipeline<C>,
    seed: f32,
}

impl<C: ContextRef> GeneratePeople<C> {
    #[inline]
    pub fn new (context: C) -> Result<Self> where C: Clone {
        const WORDS: &[u32] = include_spv!("generate_people.spv");
        let pipeline = ComputeBuilder::new(context)
            .entry(cstr!("generate_people"))
            .binding(DescriptorType::StorageBuffer, 1)
            .build(WORDS)?;

        return Ok(Self {
            pipeline,
            seed: 100f32 * thread_rng().sample::<f32, _>(OpenClosed01),
        });
    }

    #[inline]
    pub fn generate<A: DeviceAllocator> (&mut self, len: u64, usage: UsageFlags, flags: BufferFlags, memory_flags: MemoryFlags, alloc: A) -> Result<Buffer<Person, A>> where C: Clone {
        let people = Buffer::new_uninit(len, usage, flags, memory_flags, alloc)?;
        return self.call(people)
    }

    #[inline]
    pub fn call<P: DeviceAllocator>(
        &mut self,
        people: Buffer<MaybeUninit<Person>, P>
    ) -> Result<Buffer<Person, P>> {
        let set: &DescriptorSet = self.pipeline.sets().first().unwrap();
        let people_desc = set.write_descriptor(&people, 0);
        self.pipeline.sets_mut().update(&[people_desc]);

        self.pipeline.compute(..)?
            .push_contant(&self.seed)
            .dispatch(u64_to_u32(people.len()), 1, 1)?;

        std::thread::sleep(Duration::from_secs(2));
        // let mut fence = Fence::new(self.pipeline.device(), FenceFlags::empty())?;
        // fence.bind_to::<_, Pool>(pool, queue, None)?;
        // fence.wait(None)?;

        self.seed = 100f32 * thread_rng().sample::<f32, _>(OpenClosed01);
        return unsafe { Ok(people.assume_init()) };
    }
}

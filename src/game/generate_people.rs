use rand::{random};
use shared::person::Person;
use std::{mem::MaybeUninit};
use vulkan::{
    alloc::{DeviceAllocator, MemoryFlags},
    buffer::{Buffer, BufferFlags, UsageFlags},
    context::{ContextRef, event::{consumer::EventConsumer, Event}, Context},
    cstr,
    descriptor::{DescriptorSet, DescriptorType},
    include_spv,
    pipeline::{ComputeBuilder, Pipeline},
    utils::u64_to_u32,
    Result,
};

pub struct GeneratePeople<C: ContextRef> {
    pipeline: Pipeline<C>,
    seed: [u32; 4],
}

impl<C: ContextRef> GeneratePeople<C> {
    #[inline]
    pub fn new(context: C) -> Result<Self>
    where
        C: Clone,
    {
        const WORDS: &[u32] = include_spv!("generate_people.spv");
        let pipeline = ComputeBuilder::new(context)
            .entry(cstr!("generate_people"))
            .binding(DescriptorType::StorageBuffer, 1)
            .build(WORDS)?;

        return Ok(Self {
            pipeline,
            seed: random(),
        });
    }

    #[inline]
    pub fn call<A: DeviceAllocator>(
        &mut self,
        len: u64,
        usage: UsageFlags,
        flags: BufferFlags,
        memory_flags: MemoryFlags,
        alloc: A,
    ) -> Result<Event<&Context, GeneratePeopleConsumer<A>>> {
        let people = Buffer::new_uninit(len, usage, flags, memory_flags, alloc)?;

        let set: &DescriptorSet = self.pipeline.sets().first().unwrap();
        let people_desc = set.write_descriptor(&people, 0);
        self.pipeline.sets_mut().update(&[people_desc]);

        let seed = core::mem::replace(&mut self.seed, random());
        let event = self.pipeline
            .compute(..)?
            .push_contant(&seed)
            .dispatch(u64_to_u32(people.len()), 1, 1)?;

        let (event, _) = event.replace(GeneratePeopleConsumer {
            result: people
        });

        return Ok(event)
    }
}

pub struct GeneratePeopleConsumer<A: DeviceAllocator> {
    result: Buffer<MaybeUninit<Person>, A>
}

unsafe impl<'a, A: DeviceAllocator> EventConsumer
    for GeneratePeopleConsumer<A>
{
    type Output = Buffer<Person, A>;

    #[inline]
    fn consume(self) -> Self::Output {
        return unsafe { self.result.assume_init() };
    }
}

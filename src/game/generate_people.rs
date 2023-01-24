use rand::random;
use shared::person::Person;
use std::{mem::MaybeUninit, pin::Pin, ops::RangeBounds};
use vulkan::{
    alloc::{DeviceAllocator, MemoryFlags},
    buffer::{Buffer, BufferFlags, UsageFlags},
    context::{
        event::{consumer::EventConsumer, Event},
        Context, ContextRef,
    },
    cstr,
    descriptor::{DescriptorSet, DescriptorType},
    include_spv,
    pipeline::{ComputeBuilder, Pipeline},
    utils::u64_to_u32,
    Result, forward_phantom,
};

pub struct GeneratePeople<C: ContextRef> {
    pipeline: Pipeline<C>,
    seed: [u32; 4],
}

impl<C: Clone + Unpin + ContextRef> GeneratePeople<C> {
    #[inline]
    pub fn new(context: C) -> Result<Self> {
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
    pub fn generate<A: DeviceAllocator>(
        &mut self,
        len: u64,
        usage: UsageFlags,
        flags: BufferFlags,
        memory_flags: MemoryFlags,
        alloc: A,
    ) -> Result<Event<Pin<C>, GeneratePeopleConsumer<A>>> {
        let people = Buffer::new_uninit(len, usage, flags, memory_flags, alloc)?;

        let set: &DescriptorSet = self.pipeline.sets().first().unwrap();
        let people_desc = set.write_descriptor(&people, .., 0);
        self.pipeline.sets_mut().update(&[people_desc]);

        let seed = core::mem::replace(&mut self.seed, random());
        let event = self.pipeline.compute_owned(..)?.push_contant(&seed).dispatch(
            u64_to_u32(people.len()),
            1,
            1,
        )?;

        let (event, _) = event.replace(GeneratePeopleConsumer { result: people });

        return Ok(event);
    }

    #[inline]
    pub fn initialize<'a, A: DeviceAllocator>(
        &mut self,
        people: &'a mut Buffer<MaybeUninit<Person>, A>,
        bounds: impl RangeBounds<u64>
    ) -> Result<Event<Pin<C>, InitializePeopleConsumer<'a, A>>> {
        let set: &DescriptorSet = self.pipeline.sets().first().unwrap();
        let people_desc = set.write_descriptor(&people, bounds, 0);
        self.pipeline.sets_mut().update(&[people_desc]);

        let seed = core::mem::replace(&mut self.seed, random());
        let event = self.pipeline.compute_owned(..)?.push_contant(&seed).dispatch(
            u64_to_u32(people.len()),
            1,
            1,
        )?;

        let (event, _) = event.replace(InitializePeopleConsumer::new());
        return Ok(event);
    }
}

pub struct GeneratePeopleConsumer<A: DeviceAllocator> {
    result: Buffer<MaybeUninit<Person>, A>,
}

unsafe impl<'a, A: DeviceAllocator> EventConsumer for GeneratePeopleConsumer<A> {
    type Output = Buffer<Person, A>;

    #[inline]
    fn consume(self) -> Self::Output {
        return unsafe { self.result.assume_init() };
    }
}

forward_phantom! {
    &'a mut Buffer<MaybeUninit<Person>, A> as pub InitializePeopleConsumer<'a, A: DeviceAllocator,>
}
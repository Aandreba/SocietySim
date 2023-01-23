use std::{mem::MaybeUninit, marker::PhantomData};
use rand::{distributions::OpenClosed01, thread_rng, Rng};
use shared::{person::Person, person_event::PersonalEvent, ExternBool};
use vulkan::{
    alloc::{DeviceAllocator, MemoryFlags},
    buffer::{Buffer, BufferFlags, UsageFlags},
    context::{event::{consumer::EventConsumer, Event}, Context, ContextRef},
    cstr,
    descriptor::{DescriptorSet, DescriptorType},
    include_spv,
    pipeline::{ComputeBuilder, Pipeline},
    utils::u64_to_u32,
    Result,
};

pub struct PersonalEvents<C: ContextRef> {
    pipeline: Pipeline<C>,
    seed: f32,
}

impl<C: ContextRef> PersonalEvents<C> {
    #[inline]
    pub fn new(context: C) -> Result<Self>
    where
        C: Clone,
    {
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
    pub fn context(&self) -> &Context {
        return self.pipeline.context();
    }

    #[inline]
    pub fn call<'a, P: Clone + DeviceAllocator, E: DeviceAllocator>(
        &'a mut self,
        people: &'a Buffer<Person, P>,
        events: &'a Buffer<PersonalEvent, E>,
    ) -> Result<Event<&'a Context, PersonalEventsConsumer<'a, C, P, E>>> {
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
        self.pipeline
            .sets_mut()
            .update(&[people_desc, events_desc, result_desc]);

        let event = self
            .pipeline
            .compute(..)?
            .push_contant(&self.seed)
            .dispatch(u64_to_u32(people.len()), u64_to_u32(events.len()), 1)?;

        let (event, _) = unsafe {
            event.replace(PersonalEventsConsumer {
                parent: self,
                result,
                _phtm: PhantomData
            })
        };

        return Ok(event);
    }
}

pub struct PersonalEventsConsumer<'a, C: ContextRef, P: DeviceAllocator, E: DeviceAllocator> {
    parent: &'a mut PersonalEvents<C>,
    result: Buffer<MaybeUninit<ExternBool>, P>,
    _phtm: PhantomData<(&'a Buffer<Person, P>, &'a Buffer<PersonalEvent, E>)>
}

unsafe impl<'a, C: ContextRef, P: DeviceAllocator, E: DeviceAllocator> EventConsumer for PersonalEventsConsumer<'a, C, P, E> {
    type Output = Buffer<ExternBool, P>;

    #[inline]
    fn consume(self) -> Self::Output {
        self.parent.seed = 100f32 * thread_rng().sample::<f32, _>(OpenClosed01);
        return unsafe { self.result.assume_init() };
    }
}

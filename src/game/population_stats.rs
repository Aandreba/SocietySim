use shared::person::{stats::PopulationStats, Person};
use std::{mem::MaybeUninit, pin::Pin};
use vulkan::{
    alloc::{DeviceAllocator},
    buffer::{Buffer},
    context::{event::Event, ContextRef},
    cstr,
    descriptor::{DescriptorSet, DescriptorType},
    include_spv,
    pipeline::{ComputeBuilder, Pipeline},
    utils::u64_to_u32,
    Result, forward_phantom,
};

pub struct CalcPopulationStats<C: ContextRef> {
    pipeline: Pipeline<C>,
}

impl<C: Clone + Unpin + ContextRef> CalcPopulationStats<C> {
    #[inline]
    pub fn new(context: C) -> Result<Self> {
        const WORDS: &[u32] = include_spv!("population_stats.spv");
        let pipeline = ComputeBuilder::new(context)
            .entry(cstr!("population_stats"))
            .binding(DescriptorType::StorageBuffer, 1)
            .binding(DescriptorType::StorageBuffer, 1)
            .build(WORDS)?;

        return Ok(Self { pipeline });
    }

    #[inline]
    pub fn call<'a, 'b, A: Clone + DeviceAllocator>(
        &mut self,
        stats: &'a Buffer<PopulationStats, &'b A>,
        people: &'a Buffer<MaybeUninit<Person>, A>,
        len: u64,
    ) -> Result<Event<Pin<C>, PopulationStatsConsumer<'a, 'b, A>>> {
        let set: &DescriptorSet = self.pipeline.sets().first().unwrap();
        let people_desc = set.write_descriptor(&people);
        let stats_desc = set.write_descriptor(&stats);
        self.pipeline.sets_mut().update(&[people_desc, stats_desc]);

        return Ok(self
            .pipeline
            .compute_owned(..)?
            .dispatch(u64_to_u32(len), 1, 1)?
            .replace(PopulationStatsConsumer::new())
            .0);
    }
}

forward_phantom! {
    (&'a Buffer<PopulationStats, &'b A>, &'a Buffer<MaybeUninit<Person>, A>) as pub PopulationStatsConsumer<'a, 'b, A: DeviceAllocator,>
}
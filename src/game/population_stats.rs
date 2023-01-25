use shared::{person::{Person}, population::{PopulationMeanStats, PopulationCountStats}};
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

pub struct CalcPopulationMeanStats<C: ContextRef> {
    pipeline: Pipeline<C>,
}

impl<C: Clone + Unpin + ContextRef> CalcPopulationMeanStats<C> {
    #[inline]
    pub fn new(context: C) -> Result<Self> {
        const WORDS: &[u32] = include_spv!("population_mean_stats.spv");
        let pipeline = ComputeBuilder::new(context)
            .entry(cstr!("population_mean_stats"))
            .binding(DescriptorType::StorageBuffer, 1)
            .binding(DescriptorType::StorageBuffer, 1)
            .build(WORDS)?;

        return Ok(Self { pipeline });
    }

    #[inline]
    pub fn call<'a, 'b, A: Clone + DeviceAllocator>(
        &mut self,
        stats: &'a Buffer<PopulationMeanStats, &'b A>,
        people: &'a Buffer<MaybeUninit<Person>, A>,
        len: u64,
    ) -> Result<Event<Pin<C>, PopulationMeanStatsConsumer<'a, 'b, A>>> {
        let set: &DescriptorSet = self.pipeline.sets().first().unwrap();
        let people_desc = set.write_descriptor(&people);
        let stats_desc = set.write_descriptor(&stats);
        self.pipeline.sets_mut().update(&[people_desc, stats_desc]);

        return Ok(self
            .pipeline
            .compute_owned(..)?
            .dispatch(u64_to_u32(len), 1, 1)?
            .replace(PopulationMeanStatsConsumer::new())
            .0);
    }
}

pub struct CalcPopulationCountStats<C: ContextRef> {
    pipeline: Pipeline<C>,
}

impl<C: Clone + Unpin + ContextRef> CalcPopulationCountStats<C> {
    #[inline]
    pub fn new(context: C) -> Result<Self> {
        const WORDS: &[u32] = include_spv!("population_count_stats.spv");
        let pipeline = ComputeBuilder::new(context)
            .entry(cstr!("population_count_stats"))
            .binding(DescriptorType::StorageBuffer, 1)
            .binding(DescriptorType::StorageBuffer, 1)
            .build(WORDS)?;

        return Ok(Self { pipeline });
    }

    #[inline]
    pub fn call<'a, 'b, A: Clone + DeviceAllocator>(
        &mut self,
        stats: &'a Buffer<PopulationCountStats, &'b A>,
        people: &'a Buffer<MaybeUninit<Person>, A>,
        len: u64,
    ) -> Result<Event<Pin<C>, PopulationCountStatsConsumer<'a, 'b, A>>> {
        let set: &DescriptorSet = self.pipeline.sets().first().unwrap();
        let people_desc = set.write_descriptor(&people);
        let stats_desc = set.write_descriptor(&stats);
        self.pipeline.sets_mut().update(&[people_desc, stats_desc]);

        return Ok(self
            .pipeline
            .compute_owned(..)?
            .dispatch(u64_to_u32(len), 1, 1)?
            .replace(PopulationCountStatsConsumer::new())
            .0);
    }
}

forward_phantom! {
    (&'a Buffer<PopulationMeanStats, &'b A>, &'a Buffer<MaybeUninit<Person>, A>) as pub PopulationMeanStatsConsumer<'a, 'b, A: DeviceAllocator,>
}

forward_phantom! {
    (&'a Buffer<PopulationCountStats, &'b A>, &'a Buffer<MaybeUninit<Person>, A>) as pub PopulationCountStatsConsumer<'a, 'b, A: DeviceAllocator,>
}
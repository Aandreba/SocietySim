use crate::game::{
    generate_people::GeneratePeople,
    population_stats::{CalcPopulationCountStats, CalcPopulationMeanStats},
};
use elor::Either;
use humansize::BINARY;
use shared::{
    person::{Person, PersonStats},
    population::{GenerationOps, PopulationCountStats},
};
use std::{
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
};
use vulkan::{
    alloc::{DeviceAllocator, MemoryFlags},
    buffer::{Buffer, BufferFlags, UsageFlags},
    context::{event::Event, ContextRef},
    physical_dev::MemoryHeapFlags,
    utils::{u64_to_u32, u64_to_usize},
    Result,
};

macro_rules! iter {
    ($self:expr) => {{
        let div = $self.people / $self.chunk_size;
        let rem = $self.people % $self.chunk_size;

        let full = $self.chunks[..u64_to_usize(div)]
            .iter()
            .map(|x| ($self.chunk_size, x));
        let part = match rem {
            0 => Either::Right(core::iter::empty()).into_same_iter(),
            rem => Either::Left(core::iter::once((rem, &$self.chunks[u64_to_usize(div)])))
                .into_same_iter(),
        };

        full.chain(part)
    }};
}

pub trait PopulationAllocator =
    Clone + DeviceAllocator where <Self as DeviceAllocator>::Context: Clone + Unpin;

struct PopulationShaders<C: ContextRef> {
    stats_mean: CalcPopulationMeanStats<C>,
    stats_count: CalcPopulationCountStats<C>,
}

pub struct Population<A: PopulationAllocator> {
    chunks: Vec<Buffer<MaybeUninit<Person>, A>>,
    shaders: PopulationShaders<A::Context>,
    chunk_size: u64,
    people: u64,
    alloc: A,
}

impl<A: PopulationAllocator> Population<A> {
    #[inline]
    pub fn new(people: u64, ops: GenerationOps, alloc: A) -> Result<Self> {
        let props = alloc.device().physical().properties();
        let chunk_size = props.max_allocation_size()
            / (props.limits().maxMemoryAllocationCount as u64
                * Buffer::<Person, A>::BYTES_PER_ELEMENT);

        let div = people / chunk_size;
        let rem = people % chunk_size;

        #[cfg(debug_assertions)]
        println!(
            "Device total free memory: {}",
            humansize::format_size(
                alloc
                    .device()
                    .physical()
                    .available_memory(MemoryHeapFlags::DEVICE_LOCAL),
                BINARY
            )
        );
        #[cfg(debug_assertions)]
        println!(
            "Max allocation size: {}",
            humansize::format_size(
                alloc
                    .device()
                    .physical()
                    .max_available_memory(MemoryHeapFlags::DEVICE_LOCAL)
                    .unwrap_or_default(),
                BINARY
            )
        );
        #[cfg(debug_assertions)]
        println!();

        let mut init = GeneratePeople::new(alloc.context())?;
        let mut events = Vec::with_capacity(u64_to_usize(div));

        for _ in 0..div {
            events.push(init.generate(
                ops,
                chunk_size,
                UsageFlags::STORAGE_BUFFER,
                BufferFlags::empty(),
                MemoryFlags::DEVICE_LOCAL,
                alloc.clone(),
            )?);
        }

        let mut chunks = Event::join_all(events)?
            .map(Buffer::into_maybe_uninit)
            .collect::<Vec<_>>();

        // todo optimize, execute in paralel with the generators
        if rem > 0 {
            let mut rem_people = Buffer::new_uninit(
                chunk_size,
                UsageFlags::STORAGE_BUFFER,
                BufferFlags::empty(),
                MemoryFlags::DEVICE_LOCAL,
                alloc.clone(),
            )?;

            init.initialize(ops, &mut rem_people, ..u64_to_u32(rem))?
                .wait()?;
            chunks.push(rem_people);
        }

        let shaders = PopulationShaders {
            stats_mean: CalcPopulationMeanStats::new(alloc.owned_context())?,
            stats_count: CalcPopulationCountStats::new(alloc.owned_context())?,
        };

        drop(init);
        return Ok(Self {
            chunks,
            chunk_size,
            shaders,
            people,
            alloc,
        });
    }

    #[inline]
    pub fn reserve(&mut self, people: u64) -> Result<()> {
        if let Some(delta) = (self.len() + people).checked_sub(self.capacity()) {
            if delta == 0 {
                return Ok(());
            }

            let div = delta / self.chunk_size;
            let rem = delta % self.chunk_size;
            let delta = div + if rem > 0 { 1 } else { 0 };

            self.chunks.reserve(u64_to_usize(delta));
            for _ in 0..delta {
                self.chunks.push(Buffer::<Person, A>::new_uninit(
                    self.chunk_size,
                    UsageFlags::STORAGE_BUFFER,
                    BufferFlags::empty(),
                    MemoryFlags::DEVICE_LOCAL,
                    self.alloc.clone(),
                )?)
            }
        }

        return Ok(());
    }

    #[inline]
    pub fn mean_stats(&mut self) -> Result<PopulationMeanStats> {
        let result = Buffer::from_sized_iter(
            [shared::population::PopulationMeanStats::default()],
            UsageFlags::STORAGE_BUFFER,
            BufferFlags::empty(),
            MemoryFlags::MAPABLE,
            &self.alloc,
        )?;

        let events = iter!(self)
            .map(|(len, people)| self.shaders.stats_mean.call(&result, people, len))
            .try_collect::<Vec<_>>()?;

        let _ = Event::join_all(events)?;
        let map = result.map(..)?;
        return Ok(PopulationMeanStats::from_stats(self, map[0]));
    }

    #[inline]
    pub fn count_stats(&mut self) -> Result<Box<PopulationCountStats>> {
        let mut result = Buffer::new_uninit(
            1,
            UsageFlags::STORAGE_BUFFER,
            BufferFlags::empty(),
            MemoryFlags::MAPABLE,
            &self.alloc,
        )?;

        #[cfg(debug_assertions)]
        println!(
            "{} size: {}",
            core::any::type_name::<PopulationCountStats>(),
            humansize::format_size(core::mem::size_of::<PopulationCountStats>(), BINARY)
        );

        let mut map = result.map_mut(..)?;
        unsafe {
            let (lhs, map, rhs) = map.align_to_mut::<u8>();
            debug_assert_eq!(lhs.len(), 0);
            debug_assert_eq!(rhs.len(), 0);
            map.fill(0);
        }
        drop(map);

        let result = unsafe { result.assume_init() };
        let events = iter!(self)
            .map(|(len, people)| self.shaders.stats_count.call(&result, people, len))
            .try_collect::<Vec<_>>()?;

        let _ = Event::join_all(events)?;
        let mut alloc = Box::new_uninit();
        let map = result.map(..)?;
        unsafe {
            core::ptr::copy_nonoverlapping(map.as_ptr(), alloc.as_mut_ptr(), 1);
        };
        drop(map);

        return unsafe { Ok(alloc.assume_init()) };
    }

    #[inline]
    pub fn len(&self) -> u64 {
        return self.people;
    }

    #[inline]
    pub fn capacity(&self) -> u64 {
        return (self.chunks.len() as u64) * self.chunk_size;
    }

    #[inline]
    pub fn allocator(&self) -> &A {
        return &self.alloc;
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PopulationMeanStats {
    males: f32,
    stats: PersonStats<f32>,
}

impl PopulationMeanStats {
    #[inline]
    pub fn from_stats<A: PopulationAllocator>(
        pops: &Population<A>,
        stats: shared::population::PopulationMeanStats,
    ) -> Self {
        const WEIGHT: f32 = 100.0 / 255.0;
        let len = pops.len() as f32;

        return Self {
            males: 100.0 * (stats.males as f32) / len,
            stats: PersonStats {
                cordiality: WEIGHT * (stats.stats.cordiality as f32) / len,
                intelligence: WEIGHT * (stats.stats.intelligence as f32) / len,
                knowledge: WEIGHT * (stats.stats.knowledge as f32) / len,
                finesse: WEIGHT * (stats.stats.finesse as f32) / len,
                gullability: WEIGHT * (stats.stats.gullability as f32) / len,
                health: WEIGHT * (stats.stats.health as f32) / len,
            },
        };
    }
}

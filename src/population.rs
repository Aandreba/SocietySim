use crate::game::generate_people::GeneratePeople;
use humansize::BINARY;
use shared::person::Person;
use std::mem::MaybeUninit;
use vulkan::{
    alloc::{DeviceAllocator, MemoryFlags},
    buffer::{Buffer, BufferFlags, UsageFlags},
    context::event::Event,
    utils::u64_to_usize,
    Result, physical_dev::MemoryHeapFlags,
};

pub struct Population<A: DeviceAllocator> {
    chunks: Vec<Buffer<MaybeUninit<Person>, A>>,
    chunk_size: u64,
    people: u64,
    alloc: A,
}

impl<A: Clone + DeviceAllocator> Population<A> {
    #[inline]
    pub fn new(people: u64, alloc: A) -> Result<Self> {
        let props = alloc.device().physical().properties();
        let chunk_size = props.max_allocation_size()
            / (props.limits().maxMemoryAllocationCount as u64
                * Buffer::<Person, A>::BYTES_PER_ELEMENT);

        let div = people / chunk_size;
        let rem = people % chunk_size;
        
        #[cfg(debug_assertions)]
        println!("Device total free memory: {}", humansize::format_size(alloc.device().physical().available_memory(MemoryHeapFlags::DEVICE_LOCAL), BINARY));
        #[cfg(debug_assertions)]
        println!("Max allocation size: {}", humansize::format_size(alloc.device().physical().max_available_memory(MemoryHeapFlags::DEVICE_LOCAL).unwrap_or_default(), BINARY));

        let mut init = GeneratePeople::new(alloc.context())?;
        let mut events = Vec::with_capacity(u64_to_usize(div));

        for _ in 0..div {
            events.push(init.generate(
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
                alloc.clone()
            )?;

            init.initialize(&mut rem_people, ..rem)?.wait()?;
            chunks.push(rem_people);
        }

        drop(init);
        return Ok(Self {
            chunks,
            chunk_size,
            people: div * chunk_size,
            alloc
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
    pub fn len(&self) -> u64 {
        return self.people;
    }

    #[inline]
    pub fn capacity(&self) -> u64 {
        return (self.chunks.len() as u64) * self.chunk_size;
    }
}

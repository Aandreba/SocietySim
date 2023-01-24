use humansize::BINARY;
use shared::person::Person;
use vulkan::{alloc::DeviceAllocator, buffer::Buffer, Result};

pub struct Population<A: DeviceAllocator> {
    chunks: Vec<Buffer<Person, A>>,
    cunhk_size: u64,
}

impl<A: DeviceAllocator> Population<A> {
    #[inline]
    pub fn new(people: u64, alloc: A) -> Result<Self> {
        let props = alloc.device().physical().properties();
        let chunk_size =
            props.max_allocation_size() / (props.limits().maxMemoryAllocationCount as u64);

        #[cfg(debug_assertions)]
        println!(
            "{} / {} = {}",
            humansize::format_size(props.max_allocation_size(), BINARY),
            humansize::format_size(props.limits().maxMemoryAllocationCount, BINARY),
            humansize::format_size(chunk_size, BINARY),
        );

        todo!()
    }
}

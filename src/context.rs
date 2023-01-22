use vulkan::{*, device::{DeviceRef, Device}, pool::{CommandPool, CommandPoolFlags, CommandBufferLevel}, queue::Queue};

pub struct Context<D: DeviceRef> {
    pub pool: CommandPool<D>,
    pub queue: Queue
}

impl<D: DeviceRef> Context<D> {
    #[inline]
    pub fn new (device: D, queue: Queue) -> Result<Self> {
        let family = device.physical().queue_families().next().unwrap();
        let pool = CommandPool::new(
            device,
            family,
            CommandPoolFlags::empty(),
            1,
            CommandBufferLevel::Primary,
        )?;

        return Ok(Self { pool, queue })
    }

    #[inline]
    pub fn device (&self) -> &Device {
        return self.pool.device()
    }

    #[inline]
    pub fn owned_device (&self) -> D where D: Clone {
        return self.pool.owned_device()
    }
}
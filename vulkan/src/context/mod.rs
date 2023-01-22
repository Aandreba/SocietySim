use crate::{
    device::Device,
    physical_dev::{PhysicalDevice, QueueFlags},
    utils::usize_to_u32,
    Entry, Result,
};
use std::{
    num::NonZeroU64,
    ptr::{addr_of, addr_of_mut},
    sync::{Mutex, TryLockError}, ops::Deref,
};
use self::command::Command;

pub trait ContextRef = Deref<Target = Context>;
pub mod command;

// https://stackoverflow.com/a/55273688
struct QueueFamily {
    flags: QueueFlags,
    queues: Vec<Mutex<NonZeroU64>>,
    pool_buffer: Mutex<[NonZeroU64; 2]>,
}

pub struct Context {
    device: Device,
    families: Box<[QueueFamily]>,
}

impl Context {
    #[inline]
    pub fn new(phy: PhysicalDevice, queues: usize) -> Result<Self> {
        let priorities = vec![1f32; queues];
        Self::with_priorities(phy, &priorities)
    }

    #[inline]
    pub fn with_priorities(phy: PhysicalDevice, priorities: &[f32]) -> Result<Self> {
        debug_assert!(priorities.iter().all(|x| (0f32..=1f32).contains(x)));

        let family_props = phy.queue_families_raw();
        let mut device = Device::builder(phy);

        let mut priorities = Vec::new();
        for props in family_props.iter() {
            if let Some(delta) = props.queueCount.checked_sub(priorities.len()) {
                priorities.reserve(delta);
                for _ in 0..delta {
                    priorities.push(1f32)
                }
            }
        }

        for (i, props) in family_props.iter().enumerate() {
            device = device
                .queues(&priorities[..(props.queueCount as usize)])
                .family_index(i)
                .build();
        }

        let mut families = Vec::with_capacity(family_props.len());
        for i in 0..family_props.len() {
            let family = &family_props[i];
            let pool = Self::create_command_pool(&device, usize_to_u32(i))?;
            let buffer = Self::create_command_buffer(&device, pool)?;
            let queues = (0..family.queueCount)
                .into_iter()
                .map(|j| {
                    let mut result = 0;
                    (Entry::get().get_device_queue)(device.id(), i, j, addr_of_mut!(result));
                    return NonZeroU64::new(result).map(Mutex::new);
                })
                .try_collect::<Vec<_>>()
                .ok_or(vk::ERROR_UNKNOWN.into())?;

            families.push(QueueFamily {
                flags: QueueFlags::from_bits_truncate(family.queueFlags),
                pool_buffer: Mutex::new((pool, buffer)),
                queues,
            });
        }

        return Ok(Self {
            device,
            families: families.into_boxed_slice(),
        });
    }
}

impl Context {
    fn create_command_pool(device: &Device, family_idx: u32) -> Result<NonZeroU64> {
        let info = vk::CommandPoolCreateInfo {
            sType: vk::STRUCTURE_TYPE_COMMAND_POOL_CREATE_INFO,
            pNext: core::ptr::null(),
            flags: 0, // up to change
            queueFamilyIndex: family_idx,
        };

        let mut result = 0;
        tri! {
            (Entry::get().create_command_pool)(device.id(), addr_of!(info), core::ptr::null(), addr_of_mut!(result))
        }

        return NonZeroU64::new(result).ok_or(vk::ERROR_UNKNOWN.into());
    }

    fn create_command_buffer(device: &Device, pool: NonZeroU64) -> Result<NonZeroU64> {
        let info = vk::CommandBufferAllocateInfo {
            sType: vk::STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO,
            pNext: core::ptr::null(),
            commandPool: pool.get(),
            level: vk::COMMAND_BUFFER_LEVEL_PRIMARY,
            commandBufferCount: 1,
        };

        let mut result = 0;
        tri! {
            (Entry::get().allocate_command_buffers)(
                device.id(),
                addr_of!(info),
                addr_of_mut!(result)
            )
        }

        return NonZeroU64::new(result).ok_or(vk::ERROR_UNKNOWN.into());
    }
}

impl Context {
    #[inline]
    pub fn device(&self) -> &Device {
        &self.device
    }

    #[inline]
    fn command(&self, flags: QueueFlags) -> Command<'_> {
        let pool = 'outer: loop {
            for queue in self.families.iter() {
                if queue.flags.contains(flags) {
                    match queue.pool_buffer.try_lock() {
                        Ok(x) => break 'outer x,
                        Err(TryLockError::Poisoned(e)) => break 'outer e.into_inner(),
                        Err(_) => {}
                    }
                }
            }
            std::thread::yield_now();
        };

        return Command::new(self, pool);
    }
}
use self::command::{Command, ComputeCommand, TransferCommand};
use crate::{
    descriptor::DescriptorSet,
    device::Device,
    error::Error,
    physical_dev::{PhysicalDevice, QueueFlags},
    pipeline::Pipeline,
    utils::usize_to_u32,
    Entry, Result,
};
use std::{
    num::NonZeroU64,
    ops::{Deref, Index, RangeBounds},
    pin::Pin,
    ptr::{addr_of, addr_of_mut},
    sync::{Mutex, TryLockError},
};

pub trait ContextRef = Deref<Target = Context>;
pub mod command;
pub mod event;

// https://stackoverflow.com/a/55273688
#[derive(Debug)]
pub(crate) struct QueueFamily {
    flags: QueueFlags,
    queues: Vec<Mutex<NonZeroU64>>,
    pool_buffer: Mutex<[NonZeroU64; 2]>,
}

#[derive(Debug)]
pub struct Context {
    device: Device,
    families: Box<[QueueFamily]>,
}

impl Context {
    #[inline]
    pub fn new(phy: PhysicalDevice) -> Result<Self> {
        let family_props = phy.queue_families_raw();

        let mut priorities = Vec::new();
        for props in family_props.iter() {
            if let Some(delta) = (props.queueCount as usize).checked_sub(priorities.len()) {
                priorities.reserve(delta);
                for _ in 0..delta {
                    priorities.push(1f32)
                }
            }
        }

        let mut device = Device::builder(phy);
        for (props, i) in family_props.iter().zip(0u32..) {
            device = device
                .queues(&priorities[..(props.queueCount as usize)])
                .family_index(i)?
                .build();
        }

        let device = device.build()?;
        let mut families = Vec::with_capacity(family_props.len());

        for i in 0..usize_to_u32(family_props.len()) {
            let family = &family_props[i as usize];
            let pool = Self::create_command_pool(&device, i)?;
            let buffer = Self::create_command_buffer(&device, pool)?;
            let queues = (0..family.queueCount)
                .into_iter()
                .map(|j| {
                    let mut result = 0;
                    (Entry::get().get_device_queue)(device.id(), i, j, addr_of_mut!(result));
                    return NonZeroU64::new(result).map(Mutex::new);
                })
                .try_collect::<Vec<_>>()
                .ok_or::<Error>(vk::ERROR_UNKNOWN.into())?;

            families.push(QueueFamily {
                flags: QueueFlags::from_bits_truncate(family.queueFlags),
                pool_buffer: Mutex::new([pool, buffer]),
                queues,
            });
        }

        return Ok(Self {
            device,
            families: families.into_boxed_slice(),
        });
    }

    #[inline]
    pub fn device(&self) -> &Device {
        &self.device
    }

    #[inline]
    fn command_by_deref<T: Deref<Target = Self>>(
        this: Pin<T>,
        flags: QueueFlags,
    ) -> Result<Command<T>> {
        let (pool, family) = 'outer: loop {
            for family in this.families.iter() {
                if family.flags.contains(flags) {
                    match family.pool_buffer.try_lock() {
                        Ok(x) => break 'outer (x, family),
                        Err(TryLockError::Poisoned(e)) => break 'outer (e.into_inner(), family),
                        Err(_) => {}
                    }
                }
            }
            std::thread::yield_now();
        };

        let family = family as *const QueueFamily;
        let pool = unsafe { core::mem::transmute(pool) };
        return unsafe { Command::new(this, family, pool) };
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
    pub fn compute_command_by_deref<
        'b,
        T: Deref<Target = Self>,
        C: ContextRef,
        R: RangeBounds<usize>,
    >(
        this: Pin<T>,
        pipeline: &'b Pipeline<C>,
        desc_sets: R,
    ) -> Result<ComputeCommand<'b, T, C>>
    where
        [DescriptorSet]: Index<R, Output = [DescriptorSet]>,
    {
        let cmd = Self::command_by_deref(this, QueueFlags::COMPUTE)?;
        return ComputeCommand::new(cmd, pipeline, desc_sets);
    }

    #[inline]
    pub fn transfer_command_by_deref<'b, T: Deref<Target = Self>>(
        this: Pin<T>,
    ) -> Result<TransferCommand<'b, T>> {
        return Self::command_by_deref(this, QueueFlags::TRANSFER).map(TransferCommand::new);
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        #[inline]
        fn drop_family(device: &Device, family: &mut QueueFamily) {
            let [pool, buffer] = match family.pool_buffer.get_mut() {
                Ok(x) => x,
                Err(e) => e.into_inner(),
            };

            (Entry::get().free_command_buffers)(device.id(), pool.get(), 1, &buffer.get());
            (Entry::get().destroy_command_pool)(device.id(), pool.get(), core::ptr::null());
        }

        self.families
            .iter_mut()
            .for_each(|x| drop_family(&self.device, x));

        (Entry::get().destroy_device)(self.device.id(), core::ptr::null());
    }
}

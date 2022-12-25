use std::{num::NonZeroU64, marker::PhantomData, ptr::{addr_of_mut, addr_of}};
use crate::{Result, Entry, queue::{Queue}};
use super::{PhysicalDevice, Family};

#[derive(Debug)]
#[repr(transparent)]
pub struct Device {
    inner: NonZeroU64
}

impl Device {
    #[inline]
    pub fn builder<'a> (parent: &'a PhysicalDevice) -> Builder<'a> {
        return Builder::new(parent)
    }
}

impl Drop for Device {
    #[inline]
    fn drop(&mut self) {
        (Entry::get().destroy_device)(self.inner.get(), core::ptr::null_mut())
    }
}

pub struct Builder<'a> {
    inner: vk::DeviceCreateInfo,
    parent: &'a PhysicalDevice,
    _phtm: PhantomData<&'a vk::PhysicalDeviceFeatures>
}

impl<'a> Builder<'a> {
    pub fn new (parent: &'a PhysicalDevice) -> Self {
        return Self {
            inner: vk::DeviceCreateInfo {
                sType: vk::STRUCTURE_TYPE_DEVICE_CREATE_INFO,
                pNext: core::ptr::null_mut(),
                flags: 0,
                queueCreateInfoCount: 0,
                pQueueCreateInfos: core::ptr::null_mut(),
                enabledLayerCount: 0, // depr
                ppEnabledLayerNames: core::ptr::null_mut(), // depr
                enabledExtensionCount: 0,
                ppEnabledExtensionNames: core::ptr::null_mut(),
                pEnabledFeatures: Box::into_raw(parent.features()).cast(),
            },
            parent,
            _phtm: PhantomData
        }
    }

    #[inline]
    pub fn flags (mut self, flags: DeviceFlags) -> Self {
        self.inner.flags = flags.bits();
        self
    }

    #[inline]
    pub fn queues (self, priorities: &'a [f32]) -> QueueBuilder<'a> {
        return QueueBuilder::new(self, priorities)
    }

    pub fn build (self) -> Result<(Device, Vec<Queue>)> {
        let entry = Entry::get();
        let mut result: vk::Device = 0;
        tri! {
            (entry.create_device)(self.parent.id(), addr_of!(self.inner), core::ptr::null_mut(), addr_of_mut!(result))
        };

        if let Some(inner) = NonZeroU64::new(result) {
            let mut queues = Vec::<Queue>::new();

            if !self.inner.pQueueCreateInfos.is_null() {
                let info = unsafe { &*self.inner.pQueueCreateInfos };
                queues.reserve(info.queueCount as usize);

                for i in 0..info.queueCount {
                    let mut queue = 0;
                    (entry.get_device_queue)(inner.get(), info.queueFamilyIndex, i, addr_of_mut!(queue));

                    if let Some(inner) = NonZeroU64::new(queue) {
                        queues.push(Queue { inner });
                    } else {
                        return Err(vk::ERROR_INITIALIZATION_FAILED.into())
                    }
                }
            }

            return Ok((Device { inner }, queues))
        }
        return Err(vk::ERROR_INITIALIZATION_FAILED.into())
    }
}

impl Drop for Builder<'_> {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            if !self.inner.pQueueCreateInfos.is_null() {
                let _ = Box::from_raw(self.inner.pQueueCreateInfos.cast_mut());
            }
            let _ = Box::from_raw(self.inner.pEnabledFeatures.cast_mut());
        }
    }
}

bitflags::bitflags! {
    #[repr(transparent)]
    pub struct DeviceFlags: vk::DeviceCreateFlags {
        const PROTECTED = vk::DEVICE_QUEUE_CREATE_PROTECTED_BIT;
    }
}

pub struct QueueBuilder<'a> {
    inner: Box<vk::DeviceQueueCreateInfo>,
    parent: Builder<'a>,
    _phtm: PhantomData<&'a [f32]>
}

impl<'a> QueueBuilder<'a> {
    #[inline]
    pub fn new (parent: Builder<'a>, priorities: &'a [f32]) -> Self {
        debug_assert!(f32::abs(priorities.iter().sum::<f32>() - 1f32) < f32::EPSILON);
        return Self {
            inner: Box::new(vk::DeviceQueueCreateInfo {
                sType: vk::STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO,
                pNext: core::ptr::null_mut(),
                flags: 0,
                queueFamilyIndex: 0,
                queueCount: u32::try_from(priorities.len()).unwrap(),
                pQueuePriorities: priorities.as_ptr(),
            }),
            parent,
            _phtm: PhantomData
        }
    }

    #[inline]
    pub fn family (mut self, family: &Family) -> Result<Self> {
        if &family.parent() != self.parent.parent {
            return Err(vk::ERROR_UNKNOWN.into())
        }

        self.inner.queueFamilyIndex = family.idx;
        return Ok(self)
    }

    #[inline]
    pub fn flags (mut self, flags: LogicalQueueFlags) -> Self {
        self.inner.flags = flags.bits();
        self
    }

    #[inline]
    pub fn priorities (mut self, p: &'a [f32]) -> Result<Self> {
        if p.len() != self.inner.queueCount as usize { 
            return Err(vk::ERROR_INITIALIZATION_FAILED.into());
        }
        self.inner.pQueuePriorities = p.as_ptr();
        Ok(self)
    }

    #[inline]
    pub fn build (mut self) -> Builder<'a> {
        self.parent.inner.queueCreateInfoCount = 1;
        let prev = core::mem::replace(&mut self.parent.inner.pQueueCreateInfos, Box::into_raw(self.inner));
        if !prev.is_null() {
            let _ = unsafe { Box::from_raw(prev.cast_mut()) };
        }
        self.parent
    }
}

bitflags::bitflags! {
    #[repr(transparent)]
    pub struct LogicalQueueFlags: vk::DeviceQueueCreateFlags {
        const PROTECTED = vk::DEVICE_QUEUE_CREATE_PROTECTED_BIT;
    }
}
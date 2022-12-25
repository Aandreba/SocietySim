use std::{num::NonZeroU64, marker::PhantomData};
use crate::{Result, Entry};
use super::PhysicalDevice;

#[repr(transparent)]
pub struct Device {
    inner: NonZeroU64
}

impl Device {
    pub fn new (parent: &PhysicalDevice) -> Result<()> {
        let entry = Entry::get();
        let mut result: vk::Device = 0;
        //(entry.create_device)(, addr_of_mut!(result));

        todo!()
    }
}

pub struct Builder<'a> {
    inner: vk::DeviceCreateInfo,
    _phtm: PhantomData<&'a ()>
}

impl<'a> Builder<'a> {
    pub const fn new () -> Self {
        return Self {
            inner: vk::DeviceCreateInfo {
                sType: vk::STRUCTURE_TYPE_DEVICE_CREATE_INFO,
                pNext: core::ptr::null_mut(),
                flags: 0,
                queueCreateInfoCount: 0,
                pQueueCreateInfos: core::ptr::null_mut(),
                enabledLayerCount: 0,
                ppEnabledLayerNames: core::ptr::null_mut(),
                enabledExtensionCount: 0,
                ppEnabledExtensionNames: core::ptr::null_mut(),
                pEnabledFeatures: core::ptr::null_mut(),
            },
            _phtm: PhantomData
        }
    }

    #[inline]
    pub fn flags (mut self, flags: DeviceFlags) -> Self {
        self.inner.flags = flags.bits();
        self
    }

    #[inline]
    pub fn queue (self) -> QueueBuilder<'a> {
        return QueueBuilder::new(self)
    }
}

bitflags::bitflags! {
    #[repr(transparent)]
    pub struct DeviceFlags: vk::DeviceCreateFlags {
        const PROTECTED = vk::DEVICE_QUEUE_CREATE_PROTECTED_BIT;
    }
}

pub struct QueueBuilder<'a> {
    inner: vk::DeviceQueueCreateInfo,
    parent: Builder<'a>,
    _phtm: PhantomData<&'a [f32]>
}

impl<'a> QueueBuilder<'a> {
    #[inline]
    pub const fn new (parent: Builder<'a>, count: u32) -> Self {
        return Self {
            inner: vk::DeviceQueueCreateInfo {
                sType: vk::STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO,
                pNext: core::ptr::null_mut(),
                flags: 0,
                queueFamilyIndex: 0,
                queueCount: count,
                pQueuePriorities: core::ptr::null(),
            },
            parent,
            _phtm: PhantomData
        }
    }

    #[inline]
    pub fn family_idx (mut self, idx: u32) -> Self {
        self.inner.queueFamilyIndex = idx;
        self
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
    pub fn build (self) -> Builder<'a> {
        self.parent.inner.queueCreateInfoCount = 1;
    }
}

bitflags::bitflags! {
    #[repr(transparent)]
    pub struct LogicalQueueFlags: vk::DeviceQueueCreateFlags {
        const PROTECTED = vk::DEVICE_QUEUE_CREATE_PROTECTED_BIT;
    }
}
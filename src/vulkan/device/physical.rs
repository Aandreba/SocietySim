use std::{num::NonZeroU64, ptr::addr_of_mut, ffi::CStr, fmt::Debug, mem::MaybeUninit};
use crate::vulkan::{vk, Entry, Result};

#[derive(Debug)]
#[repr(transparent)]
pub struct PhysicalDevice {
    inner: NonZeroU64
}

impl PhysicalDevice {
    #[inline]
    pub fn first () -> Result<PhysicalDevice> {
        Self::first_from_entry(Entry::get()?)
    }
    
    #[inline]
    pub fn get_all () -> Result<Vec<PhysicalDevice>> {
        Self::from_entry(Entry::get()?)
    }

    #[inline]
    fn from_entry (entry: &Entry) -> Result<Vec<PhysicalDevice>> {
        let mut count = 0;
        tri! {
            (entry.enumerate_physical_devices)(entry.instance.get(), addr_of_mut!(count), core::ptr::null_mut())
        };

        let mut devices = Vec::with_capacity(count as usize);
        tri! {
            (entry.enumerate_physical_devices)(entry.instance.get(), addr_of_mut!(count), devices.as_mut_ptr())
        };

        unsafe {
            debug_assert!(!devices.iter().any(|x| *x == 0));
            devices.set_len(count as usize);
            return Ok(core::mem::transmute(devices))
        }
    }

    #[inline]
    fn first_from_entry (entry: &Entry) -> Result<PhysicalDevice> {
        let mut device = MaybeUninit::uninit();
        tri! {
            (entry.enumerate_physical_devices)(entry.instance.get(), &mut 1, device.as_mut_ptr())
        };

        unsafe {
            if let Some(inner) = NonZeroU64::new(device.assume_init()) {
                return Ok(PhysicalDevice { inner })
            }
            return Err(vk::ERROR_UNKNOWN.into())
        }
    }

    #[inline]
    pub fn properties (&self) -> Result<Properties> {
        let mut props = MaybeUninit::uninit();
        (Entry::get()?.get_physical_device_properties)(self.inner.get(), props.as_mut_ptr());
        return unsafe { Ok(Properties { inner: props.assume_init() }) }
    }
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Properties {
    inner: vk::PhysicalDeviceProperties
}

impl Properties {
    #[inline]
    pub fn api_version (self) -> (u32, u32, u32) {
        return vk::get_version(self.inner.apiVersion)
    }

    #[inline]
    pub fn driver_version (self) -> (u32, u32, u32) {
        vk::get_version(self.inner.driverVersion)
    }

    #[inline]
    pub fn vendor_id (self) -> u32 {
        self.inner.vendorID
    }
    
    #[inline]
    pub fn device_id (self) -> u32 {
        self.inner.deviceID
    }

    #[inline]
    pub fn name (&self) -> &'_ CStr {
        return unsafe { CStr::from_ptr(self.inner.deviceName.as_ptr()) }
    }

    #[inline]
    pub fn ty (&self) -> Type {
        match self.inner.deviceType {
            vk::PHYSICAL_DEVICE_TYPE_INTEGRATED_GPU => Type::IntegratedGpu,
            vk::PHYSICAL_DEVICE_TYPE_DISCRETE_GPU => Type::DiscreteGpu,
            vk::PHYSICAL_DEVICE_TYPE_VIRTUAL_GPU => Type::VirtualGpu,
            vk::PHYSICAL_DEVICE_TYPE_CPU => Type::Cpu,
            vk::PHYSICAL_DEVICE_TYPE_OTHER | _ => Type::Other
        }
    }

    // TODO other
}

impl Debug for Properties {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Properties")
            .field("api_version", &self.api_version())
            .field("driver_version", &self.driver_version())
            .field("vendor_id", &self.vendor_id())
            .field("device_id", &self.device_id())
            .field("name", &self.name())
            .field("ty", &self.ty())
            .finish_non_exhaustive()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum Type {
    /// The device does not match any other available types.
    Other = vk::PHYSICAL_DEVICE_TYPE_OTHER,
    /// The device is typically one embedded in or tightly coupled with the host.
    IntegratedGpu = vk::PHYSICAL_DEVICE_TYPE_INTEGRATED_GPU,
    /// The device is typically a separate processor connected to the host via an interlink.
    DiscreteGpu = vk::PHYSICAL_DEVICE_TYPE_DISCRETE_GPU,
    /// The device is typically a virtual node in a virtualization environment
    VirtualGpu = vk::PHYSICAL_DEVICE_TYPE_VIRTUAL_GPU,
    /// The device is typically running on the same processors as the host.
    Cpu = vk::PHYSICAL_DEVICE_TYPE_CPU,
}
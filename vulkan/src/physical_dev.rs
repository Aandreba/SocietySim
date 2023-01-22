use crate::{vk, Entry, Result};
use std::{
    ffi::{CStr},
    fmt::Debug,
    hash::Hash,
    marker::PhantomPinned,
    mem::MaybeUninit,
    num::NonZeroU64,
    pin::Pin,
    ptr::addr_of_mut,
    sync::Arc, ops::Deref,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct PhysicalDevice {
    inner: NonZeroU64,
}

impl PhysicalDevice {
    #[inline]
    pub fn first() -> Result<PhysicalDevice> {
        Self::first_from_entry(Entry::get())
    }

    #[inline]
    pub fn get_all() -> Result<Vec<PhysicalDevice>> {
        Self::from_entry(Entry::get())
    }

    #[inline]
    fn from_entry(entry: &Entry) -> Result<Vec<PhysicalDevice>> {
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
            return Ok(core::mem::transmute(devices));
        }
    }

    #[inline]
    fn first_from_entry(entry: &Entry) -> Result<PhysicalDevice> {
        let mut device = MaybeUninit::uninit();

        match (entry.enumerate_physical_devices)(entry.instance.get(), &mut 1, device.as_mut_ptr())
        {
            vk::SUCCESS | vk::INCOMPLETE => {}
            e => return Err(e.into()),
        }

        unsafe {
            if let Some(inner) = NonZeroU64::new(device.assume_init()) {
                return Ok(PhysicalDevice { inner });
            }
            return Err(vk::ERROR_UNKNOWN.into());
        }
    }

    #[inline]
    pub fn id(self) -> u64 {
        return self.inner.get();
    }

    #[inline]
    pub fn properties(self) -> Pin<Arc<Properties>> {
        let mut props_arc = Arc::<Properties>::new(Properties {
            props: vk::PhysicalDeviceProperties2 {
                sType: vk::STRUCTURE_TYPE_PHYSICAL_DEVICE_PROPERTIES_2,
                pNext: core::ptr::null_mut(),
                properties: unsafe { #[allow(invalid_value)] MaybeUninit::uninit().assume_init() },
            },

            maintainence: vk::PhysicalDeviceMaintenance3Properties {
                sType: vk::STRUCTURE_TYPE_PHYSICAL_DEVICE_MAINTENANCE_3_PROPERTIES,
                pNext: core::ptr::null_mut(),
                maxPerSetDescriptors: 0,
                maxMemoryAllocationSize: 0,
            },

            _pin: PhantomPinned,
        });

        unsafe {
            let props = Arc::get_mut_unchecked(&mut props_arc);
            props.props.pNext = addr_of_mut!(props.maintainence).cast();

            (Entry::get().get_physical_device_properties2)(self.inner.get(), addr_of_mut!(props.props));
            return Pin::new_unchecked(props_arc);
        }
    }

    #[inline]
    pub fn features(self) -> Box<Features> {
        let mut features = Box::<Features>::new_uninit();
        (Entry::get().get_physical_device_features)(self.inner.get(), features.as_mut_ptr().cast());
        return unsafe { features.assume_init() };
    }

    pub fn queue_families_raw (self) -> Vec<vk::QueueFamilyProperties> {
        let mut count = 0;
        (Entry::get().get_physical_device_queue_family_properties)(
            self.inner.get(),
            addr_of_mut!(count),
            core::ptr::null_mut(),
        );

        let mut result = Vec::with_capacity(count as usize);
        (Entry::get().get_physical_device_queue_family_properties)(
            self.inner.get(),
            addr_of_mut!(count),
            result.as_mut_ptr(),
        );
        unsafe { result.set_len(count as usize) }

        return result;
    }
    
    #[inline]
    pub fn queue_families(self) -> impl Iterator<Item = QueueFamily> {
        return self.queue_families_raw()
            .into_iter()
            .enumerate()
            .map(move |(idx, inner)| QueueFamily {
                idx: idx as u32,
                inner,
                parent: self,
            });
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

#[derive(Clone)]
#[repr(C)]
pub struct Properties {
    props: vk::PhysicalDeviceProperties2,
    maintainence: vk::PhysicalDeviceMaintenance3Properties,
    _pin: PhantomPinned,
}

impl Properties {
    #[inline]
    pub fn api_version(&self) -> (u32, u32, u32) {
        vk::get_version(self.props.properties.apiVersion)
    }

    #[inline]
    pub fn driver_version(&self) -> (u32, u32, u32) {
        vk::get_version(self.props.properties.driverVersion)
    }

    #[inline]
    pub fn vendor_id(&self) -> u32 {
        self.props.properties.vendorID
    }

    #[inline]
    pub fn device_id(&self) -> u32 {
        self.props.properties.deviceID
    }

    #[inline]
    pub fn name(&self) -> &'_ CStr {
        return unsafe { CStr::from_ptr(self.props.properties.deviceName.as_ptr()) };
    }

    #[inline]
    pub fn ty(&self) -> Type {
        match self.props.properties.deviceType {
            vk::PHYSICAL_DEVICE_TYPE_INTEGRATED_GPU => Type::IntegratedGpu,
            vk::PHYSICAL_DEVICE_TYPE_DISCRETE_GPU => Type::DiscreteGpu,
            vk::PHYSICAL_DEVICE_TYPE_VIRTUAL_GPU => Type::VirtualGpu,
            vk::PHYSICAL_DEVICE_TYPE_CPU => Type::Cpu,
            vk::PHYSICAL_DEVICE_TYPE_OTHER | _ => Type::Other,
        }
    }

    #[inline]
    pub fn limits(&self) -> &vk::PhysicalDeviceLimits {
        return &self.props.properties.limits;
    }

    #[inline]
    pub fn as_raw (&self) -> &vk::PhysicalDeviceProperties {
        return &self.props.properties
    }

    #[inline]
    pub fn max_descriptors_per_set(&self) -> u32 {
        return self.maintainence.maxPerSetDescriptors;
    }

    #[inline]
    pub fn max_allocation_size(&self) -> u64 {
        return self.maintainence.maxMemoryAllocationSize;
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

#[derive(Clone)]
#[repr(transparent)]
pub struct Features {
    inner: vk::PhysicalDeviceFeatures,
}

impl Features {
    #[inline]
    pub fn into_raw(self) -> vk::PhysicalDeviceFeatures {
        return self.inner;
    }
}

impl Deref for Features {
    type Target = vk::PhysicalDeviceFeatures;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[derive(Clone, Copy)]
pub struct QueueFamily {
    idx: u32,
    inner: vk::QueueFamilyProperties,
    parent: PhysicalDevice,
}

impl PartialEq for QueueFamily {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.idx == other.idx && self.parent == other.parent
    }
}

impl QueueFamily {
    #[inline]
    pub fn parent(self) -> PhysicalDevice {
        return self.parent;
    }

    #[inline]
    pub fn idx(self) -> u32 {
        return self.idx;
    }

    #[inline]
    pub fn queue_flags(&self) -> QueueFlags {
        #[cfg(debug_assertions)]
        return QueueFlags::from_bits(self.inner.queueFlags).unwrap();
        #[cfg(not(debug_assertions))]
        return unsafe { QueueFlags::from_bits_unchecked(self.inner.queueFlags) };
    }
}

impl Debug for QueueFamily {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Family")
            .field("idx", &self.idx)
            .field("queue_flags", &self.queue_flags())
            .finish_non_exhaustive()
    }
}

bitflags::bitflags! {
    #[repr(transparent)]
    pub struct QueueFlags: vk::QueueFlags {
        /// Queue supports graphics operations
        const GRAPHICS = vk::QUEUE_GRAPHICS_BIT;
        /// Queue supports compute operations
        const COMPUTE = vk::QUEUE_COMPUTE_BIT;
        /// Queue supports transfer operations
        const TRANSFER = vk::QUEUE_TRANSFER_BIT;
        /// Queue supports sparse resource memory management operations
        const SPARSE_BINDING = vk::QUEUE_SPARSE_BINDING_BIT;
        /// Queues may support protected operations
        const PROTECTED = vk::QUEUE_PROTECTED_BIT;
        const VIDEO_DECODE_KHR = vk::QUEUE_VIDEO_DECODE_BIT_KHR;
        const VIDEO_ENCODE_KHR = vk::QUEUE_VIDEO_ENCODE_BIT_KHR;
        const OPTICAL_FLOW_NV = vk::QUEUE_OPTICAL_FLOW_BIT_NV;
    }
}

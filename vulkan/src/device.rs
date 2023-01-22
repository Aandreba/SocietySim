use std::{num::NonZeroU64, marker::PhantomData, ptr::{addr_of_mut, addr_of, NonNull}, hash::Hash, ffi::CStr, pin::Pin};
use crate::{Result, Entry, physical_dev::{PhysicalDevice, QueueFamily}, utils::usize_to_u32};

#[derive(Debug)]
pub struct Device {
    inner: NonZeroU64,
    parent: PhysicalDevice
}

impl PartialEq for Device {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner && self.parent == other.parent
    }
}

impl Hash for Device {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.inner.hash(state);
        self.parent.hash(state);
    }
}

impl Eq for Device {}

impl Device {
    #[inline]
    pub fn builder<'a> (parent: PhysicalDevice) -> Builder<'a> {
        return Builder::new(parent)
    }

    #[inline]
    pub fn id (&self) -> u64 {
        return self.inner.get()
    }

    #[inline]
    pub fn physical (&self) -> PhysicalDevice {
        return self.parent
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
    parent: PhysicalDevice,
    queue_infos: Pin<Vec<vk::DeviceQueueCreateInfo>>,
    _phtm: PhantomData<(&'a vk::PhysicalDeviceFeatures, &'a CStr)>
}

impl<'a> Builder<'a> {
    pub fn new (parent: PhysicalDevice) -> Self {
        return Self {
            inner: vk::DeviceCreateInfo {
                sType: vk::STRUCTURE_TYPE_DEVICE_CREATE_INFO,
                pNext: core::ptr::null_mut(),
                flags: 0,
                queueCreateInfoCount: 0, // later
                pQueueCreateInfos: core::ptr::null_mut(), // later
                enabledLayerCount: 0, // depr
                ppEnabledLayerNames: core::ptr::null_mut(), // depr
                enabledExtensionCount: 0,
                ppEnabledExtensionNames: core::ptr::null_mut(),
                pEnabledFeatures: Box::into_raw(parent.features()).cast(),
            },
            parent,
            queue_infos: Pin::new(Vec::new()),
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

    #[inline]
    pub fn extensions<I: IntoIterator<Item = &'a CStr>> (mut self, iter: I) -> Self {
        if self.inner.enabledExtensionCount > 0 && !self.inner.ppEnabledExtensionNames.is_null() {
            unsafe {
                let _ = Box::from_raw(core::slice::from_raw_parts_mut(
                    self.inner.ppEnabledExtensionNames.cast_mut(),
                    self.inner.enabledExtensionCount as usize
                ));
            }
        }

        let ext = iter.into_iter().map(CStr::as_ptr).collect::<Box<[_]>>();
        self.inner.enabledExtensionCount = usize_to_u32(ext.len());
        self.inner.ppEnabledExtensionNames = Box::into_raw(ext).cast();
        self
    }

    pub fn build (mut self) -> Result<Device> {
        let entry = Entry::get();

        self.inner.queueCreateInfoCount = usize_to_u32(self.queue_infos.len());
        self.inner.pQueueCreateInfos = self.queue_infos.as_ptr();

        let mut result: vk::Device = 0;
        tri! {
            (entry.create_device)(self.parent.id(), addr_of!(self.inner), core::ptr::null_mut(), addr_of_mut!(result))
        };

        if let Some(inner) = NonZeroU64::new(result) {
            return Ok(Device { inner, parent: self.parent })
        }

        return Err(vk::ERROR_UNKNOWN.into())
    }
}

impl Drop for Builder<'_> {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            if self.inner.enabledExtensionCount > 0 && !self.inner.ppEnabledExtensionNames.is_null() {
                let _ = Box::from_raw(core::slice::from_raw_parts_mut(
                    self.inner.ppEnabledExtensionNames.cast_mut(),
                    self.inner.enabledExtensionCount as usize
                ));
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
    inner: NonNull<vk::DeviceQueueCreateInfo>,
    parent: Builder<'a>,
    _phtm: PhantomData<&'a [f32]>
}

impl<'a> QueueBuilder<'a> {
    #[inline]
    pub fn new (mut parent: Builder<'a>, priorities: &'a [f32]) -> Self {
        debug_assert!(f32::abs(priorities.iter().sum::<f32>() - 1f32) < f32::EPSILON);
        parent.queue_infos.push(vk::DeviceQueueCreateInfo {
            sType: vk::STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO,
            pNext: core::ptr::null_mut(),
            flags: 0,
            queueFamilyIndex: 0,
            queueCount: usize_to_u32(priorities.len()),
            pQueuePriorities: priorities.as_ptr(),
        });
        
        return Self {
            inner: unsafe { NonNull::new_unchecked(parent.queue_infos.last_mut().unwrap_unchecked()) },
            parent,
            _phtm: PhantomData
        }
    }

    #[inline]
    fn inner (&mut self) -> &mut vk::DeviceQueueCreateInfo {
        unsafe { self.inner.as_mut() }
    }

    #[inline]
    pub fn family (mut self, family: &QueueFamily) -> Result<Self> {
        if family.parent() != self.parent.parent {
            return Err(vk::ERROR_UNKNOWN.into())
        }

        self.inner().queueFamilyIndex = family.idx();
        return Ok(self)
    }

    #[inline]
    pub(crate) fn family_index (mut self, family: u32) -> Result<Self> {
        self.inner().queueFamilyIndex = family;
        return Ok(self)
    }


    #[inline]
    pub fn priorities (mut self, p: &'a [f32]) -> Result<Self> {
        if p.len() != self.inner().queueCount as usize { 
            return Err(vk::ERROR_INITIALIZATION_FAILED.into());
        }
        self.inner().pQueuePriorities = p.as_ptr();
        Ok(self)
    }

    #[inline]
    pub fn build (self) -> Builder<'a> {
        self.parent
    }
}
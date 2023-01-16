#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(feature = "alloc", feature(allocator_api))]
#![feature(is_some_and, type_alias_impl_trait, ptr_metadata, new_uninit, trait_alias)]

//! https://vulkan-tutorial.com/

macro_rules! tri {
    ($($e:expr);+) => {
        $(
            match $e {
                $crate::vk::SUCCESS => {},
                e => return Err(Into::into(e))
            }
        )+
    };
}

macro_rules! flat_mod {
    ($($i:ident),+) => {
        $(
            mod $i;
            pub use $i::*;
        )+
    };
}

macro_rules! cstr {
    ($l:literal) => {
        core::ffi::CStr::from_bytes_with_nul_unchecked(
            concat!($l, "\0").as_bytes()
        )
    };
}

pub(crate) extern crate vulkan_bindings as vk;

pub mod error;
pub mod physical_dev;
pub mod device;
pub mod queue;
pub mod shader;
pub mod buffer;
pub mod alloc;
pub mod utils;
pub mod pipeline;
pub mod descriptor;
pub mod pool;

//flat_mod! { alloc }

pub type Result<T> = ::core::result::Result<T, error::Error>;

use std::{marker::{PhantomData}, ffi::{CStr, OsStr, c_char}, ptr::{addr_of, addr_of_mut}, mem::transmute, num::NonZeroU64, fmt::Debug};
use libloading::{Library};
use utils::usize_to_u32;
use vulkan_bindings::make_version;

#[cfg(windows)]
const LIB_PATH: &str = "vulkan-1.dll";
#[cfg(all(
    unix,
    not(any(target_os = "macos", target_os = "ios", target_os = "android"))
))]
const LIB_PATH: &str = "libvulkan.so.1";
#[cfg(target_os = "android")]
const LIB_PATH: &str = "libvulkan.so";
#[cfg(any(target_os = "macos", target_os = "ios"))]
const LIB_PATH: &str = "libvulkan.dylib";

static mut CURRENT_ENTRY: Option<Entry> = None;

const ENTRY_POINT: &[u8] = b"vkGetInstanceProcAddr\0";
const CREATE_INSTANCE: &CStr = unsafe { cstr!("vkCreateInstance") };

proc::entry! {
    "vkEnumeratePhysicalDevices",
    "vkGetPhysicalDeviceProperties",
    "vkCreateDevice",
    "vkGetPhysicalDeviceQueueFamilyProperties",
    "vkGetPhysicalDeviceFeatures",
    "vkGetDeviceQueue",
    "vkCreateShaderModule",
    "vkCreateBuffer",
    "vkGetBufferMemoryRequirements",
    "vkGetPhysicalDeviceMemoryProperties",
    "vkAllocateMemory",
    "vkMapMemory",
    "vkUnmapMemory",
    "vkBindBufferMemory",
    "vkCreateDescriptorSetLayout",
    "vkCreatePipelineLayout",
    "vkCreatePipelineCache",
    "vkCreateComputePipelines",
    "vkCreateDescriptorPool",
    "vkAllocateDescriptorSets",
    "vkUpdateDescriptorSets",
    "vkCreateCommandPool",
    "vkAllocateCommandBuffers",
    // Destructors
    "vkDestroyInstance",
    "vkDestroyDevice",
    "vkDestroyShaderModule",
    "vkDestroyBuffer",
    "vkDestroyDescriptorSetLayout",
    "vkDestroyPipelineLayout",
    "vkDestroyPipelineCache",
    "vkDestroyPipeline",
    "vkDestroyDescriptorPool",
    "vkDestroyCommandPool",
    "vkFreeDescriptorSets",
    "vkFreeMemory",
    "vkFreeCommandBuffers",
}

impl Entry {
    #[inline]
    pub const unsafe fn builder<'a> (api_major: u32, api_minor: u32, api_patch: u32) -> Builder<'a> {
        return Builder::new(api_major, api_minor, api_patch);
    }

    #[inline]
    pub fn get () -> &'static Self {
        unsafe {
            #[cfg(debug_assertions)]
            return CURRENT_ENTRY.as_ref().unwrap();
            #[cfg(not(debug_assertions))]
            return Ok(CURRENT_ENTRY.as_ref().unwrap_unchecked());
        }
    }
}

impl Debug for Entry {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Entry").finish_non_exhaustive()
    }
}

impl Drop for Entry {
    #[inline]
    fn drop(&mut self) {
        (self.destroy_instance)(self.instance.get(), core::ptr::null_mut())
    }
}

pub struct Builder<'a> {
    app: vk::ApplicationInfo,
    instance: vk::InstanceCreateInfo,
    _phtm: PhantomData<&'a CStr>
}

impl<'a> Builder<'a> {
    #[inline]
    pub const unsafe fn new (api_major: u32, api_minor: u32, api_patch: u32) -> Self {
        return Self {
            app: vulkan_bindings::ApplicationInfo {
                sType: vulkan_bindings::STRUCTURE_TYPE_APPLICATION_INFO,
                pNext: core::ptr::null_mut(),
                pApplicationName: core::ptr::null_mut(),
                applicationVersion: 0,
                pEngineName: core::ptr::null_mut(),
                engineVersion: 0,
                apiVersion: make_version(api_major, api_minor, api_patch),
            },

            instance: vk::InstanceCreateInfo {
                sType: vk::STRUCTURE_TYPE_INSTANCE_CREATE_INFO,
                pNext: core::ptr::null_mut(),
                flags: InstanceFlags::CREATE_ENUMERATE_PORTABILITY_BIT_KHR.bits(),
                pApplicationInfo: core::ptr::null_mut(),
                enabledLayerCount: 0,
                ppEnabledLayerNames: core::ptr::null_mut(),
                enabledExtensionCount: 0,
                ppEnabledExtensionNames: core::ptr::null_mut(),
            },

            _phtm: PhantomData
        }
    }

    #[inline]
    pub fn application (mut self, name: &'a CStr, major: u32, minor: u32, patch: u32) -> Self {
        self.app.pApplicationName = name.as_ptr();
        self.app.applicationVersion = make_version(major, minor, patch);
        self
    }

    #[inline]
    pub fn engine (mut self, name: &'a CStr, major: u32, minor: u32, patch: u32) -> Self {
        self.app.pEngineName = name.as_ptr();
        self.app.engineVersion = make_version(major, minor, patch);
        self
    }

    #[inline]
    pub fn flags (mut self, flags: InstanceFlags) -> Self {
        self.instance.flags = flags.bits();
        self
    }

    pub fn extensions<I: IntoIterator<Item = &'a CStr>> (mut self, ext: I) -> Self {
        let ext = ext.into_iter().map(CStr::as_ptr).collect::<Box<[_]>>();
        let (ptr, len) = Box::into_raw(ext).to_raw_parts();

        self.instance.enabledExtensionCount = usize_to_u32(len);
        self.instance.ppEnabledExtensionNames = ptr.cast::<*const c_char>();
        self
    }

    pub fn layers<I: IntoIterator<Item = &'a CStr>> (mut self, layers: I) -> Self {
        let layers = layers.into_iter().map(CStr::as_ptr).collect::<Box<[_]>>();
        let (ptr, len) = Box::into_raw(layers).to_raw_parts();

        self.instance.enabledLayerCount = usize_to_u32(len);
        self.instance.ppEnabledLayerNames = ptr.cast::<*const c_char>();
        self
    }

    #[inline]
    pub unsafe fn build (self) -> Result<&'static Entry> {
        self.build_in(LIB_PATH)
    }

    pub unsafe fn build_in (self, path: impl AsRef<OsStr>) -> Result<&'static Entry> {
        const NULL_INSTANCE: vk::Instance = 0;

        // TODO EXTENSIONS (https://vulkan-tutorial.com/Drawing_a_triangle/Setup/Instance)
        let mut info = self.instance;
        info.pApplicationInfo = addr_of!(self.app);

        let lib = Library::new(path)?;
        let get_instance_proc_addr = lib.get::<vk::FnGetInstanceProcAddr>(ENTRY_POINT)?.into_raw();

        let create_instance: vk::FnCreateInstance = transmute(get_instance_proc_addr(NULL_INSTANCE, CREATE_INSTANCE.as_ptr()));
        let mut instance: vk::Instance = 0;
        tri! {
            (create_instance)(addr_of!(info), core::ptr::null_mut(), addr_of_mut!(instance))
        }

        if let Some(instance) = NonZeroU64::new(instance) {
            CURRENT_ENTRY = Some(Entry::new(instance, lib, create_instance, get_instance_proc_addr));
            return Ok(Entry::get())
        }

        return Err(vk::ERROR_INITIALIZATION_FAILED.into())
    }
}

impl Drop for Builder<'_> {
    #[inline]
    fn drop(&mut self) {
        if !self.instance.ppEnabledExtensionNames.is_null() {
            let ptr = core::ptr::slice_from_raw_parts_mut(self.instance.ppEnabledExtensionNames.cast_mut(), self.instance.enabledExtensionCount as usize);
            let _ = unsafe { Box::from_raw(ptr) };
        }

        if !self.instance.ppEnabledLayerNames.is_null() {
            let ptr = core::ptr::slice_from_raw_parts_mut(self.instance.ppEnabledLayerNames.cast_mut(), self.instance.enabledLayerCount as usize);
            let _ = unsafe { Box::from_raw(ptr) };
        }
    }
}

bitflags::bitflags! {
    #[repr(transparent)]
    #[non_exhaustive]
    pub struct InstanceFlags: vk::InstanceCreateFlagBits {
        const CREATE_ENUMERATE_PORTABILITY_BIT_KHR = vk::INSTANCE_CREATE_ENUMERATE_PORTABILITY_BIT_KHR;
    }
}
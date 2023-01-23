#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(feature = "alloc", feature(allocator_api))]
#![feature(get_mut_unchecked, iterator_try_collect, is_some_and, layout_for_ptr, type_alias_impl_trait, ptr_metadata, new_uninit, trait_alias, pointer_byte_offsets)]

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

pub(crate) extern crate vulkan_bindings as vk;

pub mod error;
pub mod physical_dev;
pub mod device;
pub mod shader;
pub mod buffer;
pub mod alloc;
pub mod utils;
pub mod pipeline;
pub mod descriptor;
pub mod sync;
pub mod context;
//pub mod shared;

//flat_mod! { alloc }

pub type Result<T> = ::core::result::Result<T, error::Error>;
pub use proc::{include_spv, cstr};
use vk::get_version;

use std::{marker::{PhantomData}, ffi::{CStr, OsStr, c_char}, ptr::{addr_of, addr_of_mut}, mem::transmute, num::NonZeroU64, fmt::Debug, hash::Hash};
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
const LIB_PATH: &str = "libMoltenVK.dylib";

#[cfg(not(debug_assertions))]
static mut CURRENT_ENTRY: core::mem::MaybeUninit<Entry> = core::mem::MaybeUninit::uninit();
#[cfg(debug_assertions)]
static mut CURRENT_ENTRY: Option<Entry> = None;

const ENTRY_POINT: &[u8] = b"vkGetInstanceProcAddr\0";
const CREATE_INSTANCE: &CStr = cstr!("vkCreateInstance");

proc::entry! {
    "vkEnumeratePhysicalDevices",
    "vkGetPhysicalDeviceProperties",
    "vkGetPhysicalDeviceProperties2",
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
    "vkBeginCommandBuffer",
    "vkEndCommandBuffer",
    "vkCmdBindPipeline",
    "vkCmdBindDescriptorSets",
    "vkCmdDispatch",
    "vkCreateFence",
    "vkCreateSemaphore",
    "vkQueueSubmit",
    "vkWaitForFences",
    "vkEnumerateInstanceExtensionProperties",
    "vkCmdPushConstants",
    "vkCreateEvent",
    "vkSetEvent",
    "vkResetEvent",
    "vkGetEventStatus",
    "vkDestroyEvent",
    "vkCmdSetEvent",
    "vkCmdResetEvent",
    "vkResetFences",
    "vkGetFenceStatus",
    "vkQueueBindSparse",
    "vkGetMemoryHostPointerPropertiesEXT",
    "vkCmdCopyBuffer",
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
    "vkDestroyFence",
    "vkDestroySemaphore",
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
            return Ok(CURRENT_ENTRY.assume_init_ref());
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

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct ExtensionProperty {
    inner: vk::ExtensionProperties
}

impl PartialEq for ExtensionProperty {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.name() == other.name() && self.inner.specVersion == other.inner.specVersion
    }
}

impl Hash for ExtensionProperty {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name().hash(state);
        self.inner.specVersion.hash(state);
    }
}

impl Debug for ExtensionProperty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExtensionProperty")
            .field("name", &self.name())
            .field("version", &self.version())
            .finish()
    }
}

impl ExtensionProperty {
    #[inline]
    pub fn name (&self) -> &CStr {
        return unsafe { CStr::from_ptr(self.inner.extensionName.as_ptr()) }
    }

    #[inline]
    pub fn version (&self) -> (u32, u32, u32) {
        return get_version(self.inner.specVersion)
    }
}

pub fn extension_props() -> Result<Vec<ExtensionProperty>> {
    let entry = crate::Entry::get();

    let mut len = 0;
    tri! {
        (entry.enumerate_instance_extension_properties)(core::ptr::null(), addr_of_mut!(len), core::ptr::null_mut())
    }

    let mut result = Vec::<ExtensionProperty>::with_capacity(len as usize);
    tri! {
        (entry.enumerate_instance_extension_properties)(core::ptr::null(), addr_of_mut!(len), result.as_mut_ptr().cast())
    }
    unsafe { result.set_len(len as usize) };

    return Ok(result)
}

bitflags::bitflags! {
    #[repr(transparent)]
    #[non_exhaustive]
    pub struct InstanceFlags: vk::InstanceCreateFlagBits {
        const CREATE_ENUMERATE_PORTABILITY_BIT_KHR = vk::INSTANCE_CREATE_ENUMERATE_PORTABILITY_BIT_KHR;
    }
}
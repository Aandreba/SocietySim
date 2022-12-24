//! https://vulkan-tutorial.com/

pub(crate) extern crate vulkan_bindings as vk;
pub mod error;

pub type Result<T> = ::core::result::Result<T, error::Error>;

use std::{marker::{PhantomData}, ffi::{CStr, OsStr}, ptr::{addr_of, addr_of_mut}, mem::transmute, num::NonZeroU64};
use libloading::{Library};
use vulkan_bindings::make_version;
use crate::cstr;

macro_rules! tri {
    ($e:expr) => {
        match $e {
            $crate::vulkan::vk::SUCCESS => {},
            e => return Err(Into::into(e))
        }
    };
}

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

const ENTRY_POINT: &[u8] = b"vkGetInstanceProcAddr\0";

pub struct Entry {
    create_instance: vk::FnCreateInstance
}

impl Entry {
    #[inline]
    pub unsafe fn load () -> Result<Self> {
        Self::load_from(LIB_PATH)
    }

    pub unsafe fn load_from (path: impl AsRef<OsStr>) -> Result<Self> {
        const NULL_INSTANCE: vk::Instance = 0;
        const CREATE_INSTANCE: &CStr = cstr!("vkCreateInstance");

        let lib = Library::new(path)?;
        let get_instance_proc_addr = lib.get::<vk::FnGetInstanceProcAddr>(ENTRY_POINT)?;

        return Ok(Self {
            create_instance: transmute(get_instance_proc_addr(NULL_INSTANCE, CREATE_INSTANCE.as_ptr()))
        })
    }
}

pub struct Instance {
    inner: NonZeroU64
}

pub struct Builder<'a> {
    app_info: vk::ApplicationInfo,
    _phtm: PhantomData<&'a CStr>
}

impl<'a> Builder<'a> {
    #[inline]
    pub const fn new (api_major: u32, api_minor: u32, api_patch: u32) -> Self {
        return Self {
            app_info: vulkan_bindings::ApplicationInfo {
                sType: vulkan_bindings::STRUCTURE_TYPE_APPLICATION_INFO,
                pNext: core::ptr::null_mut(),
                pApplicationName: core::ptr::null_mut(),
                applicationVersion: 0,
                pEngineName: core::ptr::null_mut(),
                engineVersion: 0,
                apiVersion: make_version(api_major, api_minor, api_patch),
            },
            _phtm: PhantomData
        }
    }

    #[inline]
    pub fn application (&mut self, name: &'a CStr, major: u32, minor: u32, patch: u32) -> &mut Self {
        self.app_info.pApplicationName = name.as_ptr();
        self.app_info.applicationVersion = make_version(major, minor, patch);
        self
    }

    #[inline]
    pub fn engine (&mut self, name: &'a CStr, major: u32, minor: u32, patch: u32) -> &mut Self {
        self.app_info.pEngineName = name.as_ptr();
        self.app_info.engineVersion = make_version(major, minor, patch);
        self
    }

    #[inline]
    pub fn build (&self, entry: &Entry) -> Result<Instance> {
        let info = vk::InstanceCreateInfo {
            sType: vk::STRUCTURE_TYPE_INSTANCE_CREATE_INFO,
            pNext: core::ptr::null_mut(),
            flags: 0,
            pApplicationInfo: addr_of!(self.app_info),
            enabledLayerCount: 0,
            ppEnabledLayerNames: core::ptr::null_mut(),
            enabledExtensionCount: 0,
            ppEnabledExtensionNames: core::ptr::null_mut(),
        };

        // TODO EXTENSIONS (https://vulkan-tutorial.com/Drawing_a_triangle/Setup/Instance)
        let mut instance: vk::Instance = 0;
        tri! {
            (entry.create_instance)(addr_of!(info), core::ptr::null_mut(), addr_of_mut!(instance))
        }

        if let Some(inner) = NonZeroU64::new(instance) {
            return Ok(Instance { inner })
        }

        return Err(vk::ERROR_INITIALIZATION_FAILED.into())
    }
}
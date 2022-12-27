use std::{ptr::{addr_of, addr_of_mut}, num::NonZeroU64, ffi::CStr, marker::PhantomData};
use crate::{Result, Entry, device::Device, utils::usize_to_u32, descriptor::DescriptorType};

const DEFAULT_ENTRY: &CStr = unsafe { cstr!("main") };

//#[derive(PartialEq, Eq, Hash)]
pub struct Shader<'a> {
    module: NonZeroU64,
    layout: NonZeroU64,
    device: &'a Device,
    _phtm: PhantomData<&'a CStr>
}

impl<'a> Shader<'a> {
    #[inline]
    pub fn builder (device: &'a Device, stage: ShaderStage) -> Builder<'a> {
        return Builder::new(device, stage);
    }

    #[inline]
    pub fn module (&self) -> u64 {
        return self.module.get()
    }

    #[inline]
    pub fn layout (&self) -> u64 {
        return self.layout.get()
    }

    #[inline]
    pub fn device (&self) -> &Device {
        return self.device
    }
}

impl Drop for Shader<'_> {
    #[inline]
    fn drop(&mut self) {
        let entry = Entry::get();
        (entry.destroy_shader_module)(self.device().id(), self.module.get(), core::ptr::null());
        (entry.destroy_descriptor_set_layout)(self.device().id(), self.layout.get(), core::ptr::null());
    }
}

pub struct Builder<'a> {
    pub(crate) bindings: Vec<vk::DescriptorSetLayoutBinding>,
    pub(crate) flags: LayoutCreateFlags,
    pub(crate) stage: ShaderStage,
    pub(crate) device: &'a Device,
    pub(crate) entry: &'a CStr
}

impl<'a> Builder<'a> {
    #[inline]
    pub fn new (device: &'a Device, stage: ShaderStage) -> Self {
        return Self {
            bindings: Vec::new(),
            flags: LayoutCreateFlags::empty(),
            entry: DEFAULT_ENTRY,
            stage,
            device
        }
    }

    #[inline]
    pub fn entry (mut self, entry: &'a CStr) -> Self {
        self.entry = entry;
        self
    }
    
    pub fn binding (mut self, ty: DescriptorType, count: u32) -> Self {
        // Add binding
        let info = vk::DescriptorSetLayoutBinding {
            binding: usize_to_u32(self.bindings.len()),
            descriptorType: ty as vk::DescriptorType,
            descriptorCount: count,
            stageFlags: self.stage.bits(),
            pImmutableSamplers: core::ptr::null(),
        };
        self.bindings.push(info);
        self
    }

    #[inline]
    pub fn flags (mut self, flags: LayoutCreateFlags) -> Self {
        self.flags = flags;
        self
    }

    pub fn build (mut self, words: &[u32]) -> Result<Shader<'a>> {
        let entry = Entry::get();
        let info = vk::DescriptorSetLayoutCreateInfo {
            sType: vk::STRUCTURE_TYPE_DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
            pNext: core::ptr::null(),
            flags: self.flags.bits(),
            bindingCount: usize_to_u32(self.bindings.len()),
            pBindings: self.bindings.as_ptr(),
        };

        let mut layout = 0;
        tri! {
            (entry.create_descriptor_set_layout)(self.device.id(), addr_of!(info), core::ptr::null(), addr_of_mut!(layout))
        }

        if let Some(layout) = NonZeroU64::new(layout) {
            let module = match self.build_module(entry, self.device, words) {
                Ok(x) => x,
                Err(e) => {
                    (Entry::get().destroy_descriptor_set_layout)(self.device.id(), layout.get(), core::ptr::null());
                    return Err(e)
                }
            };

            return Ok(Shader {
                module,
                layout,
                device: self.device,
                _phtm: PhantomData
            })
        }

        return Err(vk::ERROR_UNKNOWN.into())
    }

    fn build_module (&mut self, entry: &Entry, device: &'a Device, words: &[u32]) -> Result<NonZeroU64> {
        let module_info = vk::ShaderModuleCreateInfo {
            sType: vk::STRUCTURE_TYPE_SHADER_MODULE_CREATE_INFO,
            pNext: core::ptr::null_mut(),
            flags: 0,
            codeSize: words.len() * core::mem::size_of::<u32>(),
            pCode: words.as_ptr().cast(),
        };

        let mut module = 0;
        tri! {
            (entry.create_shader_module)(device.id(), addr_of!(module_info), core::ptr::null(), addr_of_mut!(module))
        };

        return NonZeroU64::new(module).ok_or(vk::ERROR_INITIALIZATION_FAILED.into());
    }
}

bitflags::bitflags! {
    #[repr(transparent)]
    pub struct ShaderStage: vk::ShaderStageFlagBits {
        const VERTEX = vk::SHADER_STAGE_VERTEX_BIT;
        const TESSELLATION_CONTROL = vk::SHADER_STAGE_TESSELLATION_CONTROL_BIT;
        const TESSELLATION_EVALUATION = vk::SHADER_STAGE_TESSELLATION_EVALUATION_BIT;
        const GEOMETRY = vk::SHADER_STAGE_GEOMETRY_BIT;
        const FRAGMENT = vk::SHADER_STAGE_FRAGMENT_BIT;
        const COMPUTE = vk::SHADER_STAGE_COMPUTE_BIT;
        const ALL_GRAPHICS = vk::SHADER_STAGE_ALL_GRAPHICS;
        const ALL = vk::SHADER_STAGE_ALL;
        const RAYGEN_KHR = vk::SHADER_STAGE_RAYGEN_BIT_KHR;
        const ANY_HIT_KHR = vk::SHADER_STAGE_ANY_HIT_BIT_KHR;
        const CLOSEST_HIT_KHR = vk::SHADER_STAGE_CLOSEST_HIT_BIT_KHR;
        const MISS_KHR = vk::SHADER_STAGE_MISS_BIT_KHR;
        const INTERSECTION_KHR = vk::SHADER_STAGE_INTERSECTION_BIT_KHR;
        const CALLABLE_KHR = vk::SHADER_STAGE_CALLABLE_BIT_KHR;
        const RAYGEN_NV = vk::SHADER_STAGE_RAYGEN_BIT_NV;
        const ANY_HIT_NV = vk::SHADER_STAGE_ANY_HIT_BIT_NV;
        const CLOSEST_HIT_NV = vk::SHADER_STAGE_CLOSEST_HIT_BIT_NV;
        const MISS_NV = vk::SHADER_STAGE_MISS_BIT_NV;
        const INTERSECTION_NV = vk::SHADER_STAGE_INTERSECTION_BIT_NV;
        const CALLABLE_NV = vk::SHADER_STAGE_CALLABLE_BIT_NV;
        const TASK_NV = vk::SHADER_STAGE_TASK_BIT_NV;
        const MESH_NV = vk::SHADER_STAGE_MESH_BIT_NV;
        const TASK_EXT = vk::SHADER_STAGE_TASK_BIT_EXT;
        const MESH_EXT = vk::SHADER_STAGE_MESH_BIT_EXT;
        const SUBPASS_SHADING_HUAWEI = vk::SHADER_STAGE_SUBPASS_SHADING_BIT_HUAWEI;
    }

    #[repr(transparent)]
    pub struct LayoutCreateFlags: vk::DescriptorSetLayoutCreateFlagBits {
        const UPDATE_AFTER_BIND_POOL = vk::DESCRIPTOR_SET_LAYOUT_CREATE_UPDATE_AFTER_BIND_POOL_BIT;
        /// Descriptors are pushed via flink:vkCmdPushDescriptorSetKHR
        const PUSH_DESCRIPTOR_KHR = vk::DESCRIPTOR_SET_LAYOUT_CREATE_PUSH_DESCRIPTOR_BIT_KHR;
        const UPDATE_AFTER_BIND_POOL_EXT = vk::DESCRIPTOR_SET_LAYOUT_CREATE_UPDATE_AFTER_BIND_POOL_BIT_EXT;
        const HOST_ONLY_POOL_VALVE = vk::DESCRIPTOR_SET_LAYOUT_CREATE_HOST_ONLY_POOL_BIT_VALVE;
        const HOST_ONLY_POOL_EXT = vk::DESCRIPTOR_SET_LAYOUT_CREATE_HOST_ONLY_POOL_BIT_EXT;
    }
}
use std::{ptr::{addr_of, addr_of_mut}, num::NonZeroU64};
use crate::{Result, Entry, device::Device};

pub struct Shader<'a> {
    module: Module<'a>,
    pub(crate) layout: NonZeroU64
}

impl<'a> Shader<'a> {
    #[inline]
    pub fn module (&self) -> &Module<'a> {
        return &self.module
    }

    #[inline]
    pub fn device (&self) -> &Device {
        return self.module.device
    }
}

impl Drop for Shader<'_> {
    #[inline]
    fn drop(&mut self) {
        (Entry::get().destroy_descriptor_set_layout)(self.device().id(), self.layout.get(), core::ptr::null());
    }
}

pub struct Builder<'a> {
    bindings: Vec<vk::DescriptorSetLayoutBinding>,
    flags: LayoutCreateFlags,
    device: &'a Device
}

impl<'a> Builder<'a> {
    #[inline]
    pub fn new (device: &'a Device) -> Self {
        return Self {
            bindings: Vec::new(),
            flags: LayoutCreateFlags::empty(),
            device
        }
    }
    
    pub fn binding (mut self, ty: BindingType, count: u32, stage: StageFlags) -> Self {
        let info = vk::DescriptorSetLayoutBinding {
            binding: u32::try_from(self.bindings.len()).unwrap(),
            descriptorType: ty as vk::DescriptorType,
            descriptorCount: count,
            stageFlags: stage.bits,
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

    pub fn build (self, bytes: &[u8]) -> Result<Shader<'a>> {
        let info = vk::DescriptorSetLayoutCreateInfo {
            sType: vk::STRUCTURE_TYPE_DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
            pNext: core::ptr::null(),
            flags: self.flags.bits(),
            bindingCount: u32::try_from(self.bindings.len()).unwrap(),
            pBindings: self.bindings.as_ptr(),
        };

        let mut layout: vk::DescriptorSetLayout = 0;
        tri! {
            (Entry::get().create_descriptor_set_layout)(self.device.id(), addr_of!(info), core::ptr::null(), addr_of_mut!(layout))
        }

        if let Some(layout) = NonZeroU64::new(layout) {
            let module = Module::from_bytes(self.device, bytes)?;
            return Ok(Shader { module, layout })
        }

        return Err(vk::ERROR_UNKNOWN.into())
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Module<'a> {
    inner: NonZeroU64,
    device: &'a Device
}

impl<'a> Module<'a> {
    #[inline]
    pub fn from_bytes (device: &'a Device, b: &[u8]) -> Result<Self> {
        let module_info = vk::ShaderModuleCreateInfo {
            sType: vk::STRUCTURE_TYPE_SHADER_MODULE_CREATE_INFO,
            pNext: core::ptr::null_mut(),
            flags: 0,
            codeSize: b.len(),
            pCode: b.as_ptr().cast(),
        };

        let mut module = 0;
        tri! {
            (Entry::get().create_shader_module)(device.id(), addr_of!(module_info), core::ptr::null(), addr_of_mut!(module))
        };

        if let Some(inner) = NonZeroU64::new(module) {
            return Ok(Self { inner, device })
        }
        return Err(vk::ERROR_INITIALIZATION_FAILED.into())
    }

    #[inline]
    pub fn id (&self) -> u64 {
        return self.inner.get()
    }

    #[inline]
    pub fn device (&self) -> &Device {
        return self.device
    }
}

impl Drop for Module<'_> {
    #[inline]
    fn drop(&mut self) {
        (Entry::get().destroy_shader_module)(self.device.id(), self.inner.get(), core::ptr::null())
    }
}

#[repr(i32)]
pub enum BindingType {
    Sampler = vk::DESCRIPTOR_TYPE_SAMPLER,
    CombinedImageSampler = vk::DESCRIPTOR_TYPE_COMBINED_IMAGE_SAMPLER,
    SampledImage = vk::DESCRIPTOR_TYPE_SAMPLED_IMAGE,
    StorageImage = vk::DESCRIPTOR_TYPE_STORAGE_IMAGE,
    UniformTexelBuffer = vk::DESCRIPTOR_TYPE_UNIFORM_TEXEL_BUFFER,
    StorageTexelBuffer = vk::DESCRIPTOR_TYPE_STORAGE_TEXEL_BUFFER,
    UniformBuffer = vk::DESCRIPTOR_TYPE_UNIFORM_BUFFER,
    StorageBuffer = vk::DESCRIPTOR_TYPE_STORAGE_BUFFER,
    UniformBufferDynamic = vk::DESCRIPTOR_TYPE_UNIFORM_BUFFER_DYNAMIC,
    StorageBufferDynamic = vk::DESCRIPTOR_TYPE_STORAGE_BUFFER_DYNAMIC,
    InputAttachment = vk::DESCRIPTOR_TYPE_INPUT_ATTACHMENT,
    InlineUniformBlock = vk::DESCRIPTOR_TYPE_INLINE_UNIFORM_BLOCK,
    AccelerationStructureKhr = vk::DESCRIPTOR_TYPE_ACCELERATION_STRUCTURE_KHR,
    AccelerationStructureNv = vk::DESCRIPTOR_TYPE_ACCELERATION_STRUCTURE_NV,
    MutableValve = vk::DESCRIPTOR_TYPE_MUTABLE_VALVE,
    SampleWeightImageQcom = vk::DESCRIPTOR_TYPE_SAMPLE_WEIGHT_IMAGE_QCOM,
    BlockMatchImageQcom = vk::DESCRIPTOR_TYPE_BLOCK_MATCH_IMAGE_QCOM
}

bitflags::bitflags! {
    #[repr(transparent)]
    pub struct StageFlags: vk::ShaderStageFlagBits {
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
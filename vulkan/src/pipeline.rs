use std::{marker::PhantomData, num::NonZeroU64, ptr::{addr_of, addr_of_mut}, ffi::CStr};
use crate::{shader::{Shader, StageFlags}, device::Device, Entry, Result};

pub struct Builder<'a, 'b> {
    layout_flags: PipelineLayoutFlags,
    cache_flags: PipelineCacheFlags,
    stage_flags: PipelineStageFlags,
    shaders: Vec<u64>,
    stages: Vec<vk::PipelineShaderStageCreateInfo>,
    device: &'a Device,
    _phtm: PhantomData<&'b [Shader<'a>]>
}

impl<'a, 'b> Builder<'a, 'b> {
    #[inline]
    pub fn new (device: &'a Device) -> Self {
        return Self {
            layout_flags: PipelineLayoutFlags::empty(),
            cache_flags: PipelineCacheFlags::empty(),
            stage_flags: PipelineStageFlags::empty(),
            shaders: Vec::new(),
            stages: Vec::new(),
            device,
            _phtm: PhantomData,
        }
    }
    
    #[inline]
    pub fn layout_flags (mut self, flags: PipelineLayoutFlags) -> Self {
        self.layout_flags = flags;
        self
    }

    #[inline]
    pub fn cache_flags (mut self, flags: PipelineLayoutFlags) -> Self {
        self.layout_flags = flags;
        self
    }

    #[inline]
    pub fn stage_flags (mut self, flags: PipelineStageFlags) -> Self {
        self.stage_flags = flags;
        self
    }

    #[inline]
    pub fn shader (mut self, shader: &'b Shader<'a>, name: &'b CStr, stage: StageFlags) -> Self {
        debug_assert_eq!(shader.device(), self.device);

        let stage = vk::PipelineShaderStageCreateInfo {
            sType: vk::STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO,
            pNext: core::ptr::null(),
            flags: self.stage_flags.bits(),
            stage: stage.bits(),
            module: shader.module().id(),
            pName: name.as_ptr(),
            pSpecializationInfo: core::ptr::null(), // todo
        };

        self.shaders.push(shader.layout.get());
        self.stages.push(stage);
        self
    }

    pub fn build (self) -> Result<Pipeline> {
        let entry = Entry::get();

        let layout_info = vk::PipelineLayoutCreateInfo {
            sType: vk::STRUCTURE_TYPE_PIPELINE_LAYOUT_CREATE_INFO,
            pNext: core::ptr::null(),
            flags: self.layout_flags.bits(),
            setLayoutCount: u32::try_from(self.shaders.len()).unwrap(),
            pSetLayouts: self.shaders.as_ptr(),
            pushConstantRangeCount: 0,
            pPushConstantRanges: core::ptr::null(),
        };

        let cache_info = vk::PipelineCacheCreateInfo {
            sType: vk::STRUCTURE_TYPE_PIPELINE_CACHE_CREATE_INFO,
            pNext: core::ptr::null(),
            flags: self.cache_flags.bits(),
            initialDataSize: 0,
            pInitialData: core::ptr::null(),
        };

        let mut layout = 0;
        let mut cache = 0;
        tri! {
            (entry.create_pipeline_layout)(self.device.id(), addr_of!(layout_info), core::ptr::null(), addr_of_mut!(layout));
            (entry.create_pipeline_cache)(self.device.id(), addr_of!(cache_info), core::ptr::null(), addr_of_mut!(cache))
        };

        if let Some((layout, cache)) = NonZeroU64::new(layout).zip(NonZeroU64::new(cache))  {
            let info = vk::ComputePipelineCreateInfo {
                sType: vk::STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO,
                pNext: todo!(),
                flags: todo!(),
                stage: todo!(),
                layout: todo!(),
                basePipelineHandle: todo!(),
                basePipelineIndex: todo!(),
            };
        }

        return Err(vk::ERROR_UNKNOWN.into())
    }
}

pub struct Pipeline {
    
}

bitflags::bitflags! {
    #[repr(transparent)]
    pub struct PipelineLayoutFlags: vk::PipelineLayoutCreateFlagBits {
        const INDEPENDENT_SETS_EXT = vk::PIPELINE_LAYOUT_CREATE_INDEPENDENT_SETS_BIT_EXT;
    }

    #[repr(transparent)]
    pub struct PipelineCacheFlags: vk::PipelineCacheCreateFlagBits {
        const EXTERNALLY_SYNCHRONIZED = vk::PIPELINE_CACHE_CREATE_EXTERNALLY_SYNCHRONIZED_BIT;
        const EXTERNALLY_SYNCHRONIZED_EXT = vk::PIPELINE_CACHE_CREATE_EXTERNALLY_SYNCHRONIZED_BIT_EXT;
    }

    #[repr(transparent)]
    pub struct PipelineStageFlags: vk::PipelineShaderStageCreateFlagBits {
        const ALLOW_VARYING_SUBGROUP_SIZE = vk::PIPELINE_SHADER_STAGE_CREATE_ALLOW_VARYING_SUBGROUP_SIZE_BIT;
        const REQUIRE_FULL_SUBGROUPS = vk::PIPELINE_SHADER_STAGE_CREATE_REQUIRE_FULL_SUBGROUPS_BIT;
        const ALLOW_VARYING_SUBGROUP_SIZE_EXT = vk::PIPELINE_SHADER_STAGE_CREATE_ALLOW_VARYING_SUBGROUP_SIZE_BIT_EXT;
        const REQUIRE_FULL_SUBGROUPS_EXT = vk::PIPELINE_SHADER_STAGE_CREATE_REQUIRE_FULL_SUBGROUPS_BIT_EXT;
    }
}
use std::{marker::PhantomData, num::NonZeroU64, ptr::{addr_of, addr_of_mut}, ffi::CStr};
use crate::{shader::{Shader, StageFlags}, Entry, Result, device::Device};

pub struct ComputeBuilder<'a, 'b> {
    flags: PipelineFlags,
    layout_flags: PipelineLayoutFlags,
    cache_flags: Option<PipelineCacheFlags>,
    shader: &'b Shader<'a>,
    stage: vk::PipelineShaderStageCreateInfo,
    _phtm: PhantomData<&'b CStr>
}

impl<'a, 'b> ComputeBuilder<'a, 'b> {
    #[inline]
    pub fn new (shader: &'b Shader<'a>, entry: &'b CStr) -> Self {
        return Self {
            flags: PipelineFlags::empty(),
            layout_flags: PipelineLayoutFlags::empty(),
            cache_flags: None,
            stage: vk::PipelineShaderStageCreateInfo {
                sType: vk::STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO,
                pNext: core::ptr::null(),
                flags: 0,
                stage: StageFlags::COMPUTE.bits(),
                module: shader.module().id(),
                pName: entry.as_ptr(),
                pSpecializationInfo: core::ptr::null(), // todo
            },
            shader,
            _phtm: PhantomData,
        }
    }

    #[inline]
    fn device (&self) -> &Device {
        return self.shader.device()
    }
    
    #[inline]
    pub fn flags (mut self, flags: PipelineFlags) -> Self {
        self.flags = flags;
        self
    }
    
    #[inline]
    pub fn layout_flags (mut self, flags: PipelineLayoutFlags) -> Self {
        self.layout_flags = flags;
        self
    }

    #[inline]
    pub fn cache_flags (mut self, flags: PipelineCacheFlags) -> Self {
        self.cache_flags = Some(flags);
        self
    }

    pub fn build_compute (self) -> Result<Pipeline<'a>> {
        let entry = Entry::get();

        // Create pipeline cache
        let mut cache = None;
        if let Some(cache_flags) = self.cache_flags {
            let cache_info = vk::PipelineCacheCreateInfo {
                sType: vk::STRUCTURE_TYPE_PIPELINE_CACHE_CREATE_INFO,
                pNext: core::ptr::null(),
                flags: cache_flags.bits(),
                initialDataSize: 0,
                pInitialData: core::ptr::null(),
            };
            let mut my_cache = 0;
            tri! {
                (entry.create_pipeline_cache)(self.device().id(), addr_of!(cache_info), core::ptr::null(), addr_of_mut!(my_cache))
            }
            if let Some(my_cache) = NonZeroU64::new(my_cache) {
                cache = Some(PipelineCache { inner: my_cache, device: self.device() });
            } else {
                return Err(vk::ERROR_UNKNOWN.into())
            }
        }

        // Create pipeline layout
        let mut my_layout = 0;
        let layout_info = vk::PipelineLayoutCreateInfo {
            sType: vk::STRUCTURE_TYPE_PIPELINE_LAYOUT_CREATE_INFO,
            pNext: core::ptr::null(),
            flags: self.layout_flags.bits(),
            setLayoutCount: 1,
            pSetLayouts: addr_of!(self.shader.layout).cast(),
            pushConstantRangeCount: 0,
            pPushConstantRanges: core::ptr::null(),
        };
        tri! { (entry.create_pipeline_layout)(self.device().id(), addr_of!(layout_info), core::ptr::null(), addr_of_mut!(my_layout)) }
        let layout;
        if let Some(my_layout) = NonZeroU64::new(my_layout) {
            layout = PipelineLayout { inner: my_layout, device: self.device() };
        } else {
            return Err(vk::ERROR_UNKNOWN.into())
        }

        // Create pipeline
        let mut pipeline = 0;
        let info = vk::ComputePipelineCreateInfo {
            sType: vk::STRUCTURE_TYPE_COMPUTE_PIPELINE_CREATE_INFO,
            pNext: core::ptr::null(),
            flags: self.flags.bits(),
            stage: self.stage,
            layout: layout.id(),
            basePipelineHandle: vk::NULL_HANDLE,
            basePipelineIndex: 0,
        };
        tri! {
            (entry.create_compute_pipelines)(
                self.device().id(),
                cache.as_ref().map_or(vk::NULL_HANDLE, PipelineCache::id),
                1,
                addr_of!(info),
                core::ptr::null(),
                addr_of_mut!(pipeline)
            )
        };

        if let Some(inner) = NonZeroU64::new(pipeline) {
            return Ok(Pipeline { inner, device: self.shader.module.device })
        }
        return Err(vk::ERROR_UNKNOWN.into())
    }
}

#[derive(Debug)]
pub struct Pipeline<'a> {
    inner: NonZeroU64,
    device: &'a Device
}

impl<'a> Pipeline<'a> {
    #[inline]
    pub fn compute<'b> (shader: &'b Shader<'a>, entry: &'b CStr) -> ComputeBuilder<'a, 'b> {
        return ComputeBuilder::new(shader, entry)
    }
}

impl Drop for Pipeline<'_> {
    #[inline]
    fn drop(&mut self) {
        (Entry::get().destroy_pipeline)(self.device.id(), self.inner.get(), core::ptr::null())
    }
}

bitflags::bitflags! {
    #[repr(transparent)]
    pub struct PipelineFlags: vk::PipelineCreateFlagBits {
        const DISABLE_OPTIMIZATION = vk::PIPELINE_CREATE_DISABLE_OPTIMIZATION_BIT;
        const ALLOW_DERIVATIVES = vk::PIPELINE_CREATE_ALLOW_DERIVATIVES_BIT;
        const DERIVATIVE = vk::PIPELINE_CREATE_DERIVATIVE_BIT;
        const VIEW_INDEX_FROM_DEVICE_INDEX = vk::PIPELINE_CREATE_VIEW_INDEX_FROM_DEVICE_INDEX_BIT;
        const DISPATCH_BASE = vk::PIPELINE_CREATE_DISPATCH_BASE;
        const FAIL_ON_PIPELINE_COMPILE_REQUIRED = vk::PIPELINE_CREATE_FAIL_ON_PIPELINE_COMPILE_REQUIRED_BIT;
        const EARLY_RETURN_ON_FAILURE = vk::PIPELINE_CREATE_EARLY_RETURN_ON_FAILURE_BIT;
        const RENDERING_FRAGMENT_SHADING_RATE_ATTACHMENT_KHR = vk::PIPELINE_CREATE_RENDERING_FRAGMENT_SHADING_RATE_ATTACHMENT_BIT_KHR;
        const RASTERIZATION_STATE_CREATE_FRAGMENT_SHADING_RATE_ATTACHMENT_KHR = vk::PIPELINE_RASTERIZATION_STATE_CREATE_FRAGMENT_SHADING_RATE_ATTACHMENT_BIT_KHR; // Backwards-compatible alias containing a typo
        const RENDERING_FRAGMENT_DENSITY_MAP_ATTACHMENT_EXT = vk::PIPELINE_CREATE_RENDERING_FRAGMENT_DENSITY_MAP_ATTACHMENT_BIT_EXT;
        const RASTERIZATION_STATE_CREATE_FRAGMENT_DENSITY_MAP_ATTACHMENT_EXT = vk::PIPELINE_RASTERIZATION_STATE_CREATE_FRAGMENT_DENSITY_MAP_ATTACHMENT_BIT_EXT; // Backwards-compatible alias containing a typo
        const VIEW_INDEX_FROM_DEVICE_INDEX_KHR = vk::PIPELINE_CREATE_VIEW_INDEX_FROM_DEVICE_INDEX_BIT_KHR;
        const DISPATCH_BASE_KHR = vk::PIPELINE_CREATE_DISPATCH_BASE_KHR;
        const RAY_TRACING_NO_NULL_ANY_HIT_SHADERS_KHR = vk::PIPELINE_CREATE_RAY_TRACING_NO_NULL_ANY_HIT_SHADERS_BIT_KHR;
        const RAY_TRACING_NO_NULL_CLOSEST_HIT_SHADERS_KHR = vk::PIPELINE_CREATE_RAY_TRACING_NO_NULL_CLOSEST_HIT_SHADERS_BIT_KHR;
        const RAY_TRACING_NO_NULL_MISS_SHADERS_KHR = vk::PIPELINE_CREATE_RAY_TRACING_NO_NULL_MISS_SHADERS_BIT_KHR;
        const RAY_TRACING_NO_NULL_INTERSECTION_SHADERS_KHR = vk::PIPELINE_CREATE_RAY_TRACING_NO_NULL_INTERSECTION_SHADERS_BIT_KHR;
        const RAY_TRACING_SKIP_TRIANGLES_KHR = vk::PIPELINE_CREATE_RAY_TRACING_SKIP_TRIANGLES_BIT_KHR;
        const RAY_TRACING_SKIP_AABBS_KHR = vk::PIPELINE_CREATE_RAY_TRACING_SKIP_AABBS_BIT_KHR;
        const RAY_TRACING_SHADER_GROUP_HANDLE_CAPTURE_REPLAY_KHR = vk::PIPELINE_CREATE_RAY_TRACING_SHADER_GROUP_HANDLE_CAPTURE_REPLAY_BIT_KHR;
        const DEFER_COMPILE_NV = vk::PIPELINE_CREATE_DEFER_COMPILE_BIT_NV;
        const CAPTURE_STATISTICS_KHR = vk::PIPELINE_CREATE_CAPTURE_STATISTICS_BIT_KHR;
        const CAPTURE_INTERNAL_REPRESENTATIONS_KHR = vk::PIPELINE_CREATE_CAPTURE_INTERNAL_REPRESENTATIONS_BIT_KHR;
        const INDIRECT_BINDABLE_NV = vk::PIPELINE_CREATE_INDIRECT_BINDABLE_BIT_NV;
        const LIBRARY_KHR = vk::PIPELINE_CREATE_LIBRARY_BIT_KHR;
        const FAIL_ON_PIPELINE_COMPILE_REQUIRED_EXT = vk::PIPELINE_CREATE_FAIL_ON_PIPELINE_COMPILE_REQUIRED_BIT_EXT;
        const EARLY_RETURN_ON_FAILURE_EXT = vk::PIPELINE_CREATE_EARLY_RETURN_ON_FAILURE_BIT_EXT;
        const RETAIN_LINK_TIME_OPTIMIZATION_INFO_EXT = vk::PIPELINE_CREATE_RETAIN_LINK_TIME_OPTIMIZATION_INFO_BIT_EXT;
        const LINK_TIME_OPTIMIZATION_EXT = vk::PIPELINE_CREATE_LINK_TIME_OPTIMIZATION_BIT_EXT;
        const RAY_TRACING_ALLOW_MOTION_NV = vk::PIPELINE_CREATE_RAY_TRACING_ALLOW_MOTION_BIT_NV;
        const COLOR_ATTACHMENT_FEEDBACK_LOOP_EXT = vk::PIPELINE_CREATE_COLOR_ATTACHMENT_FEEDBACK_LOOP_BIT_EXT;
        const DEPTH_STENCIL_ATTACHMENT_FEEDBACK_LOOP_EXT = vk::PIPELINE_CREATE_DEPTH_STENCIL_ATTACHMENT_FEEDBACK_LOOP_BIT_EXT;
        const RAY_TRACING_OPACITY_MICROMAP_EXT = vk::PIPELINE_CREATE_RAY_TRACING_OPACITY_MICROMAP_BIT_EXT;
        const NO_PROTECTED_ACCESS_EXT = vk::PIPELINE_CREATE_NO_PROTECTED_ACCESS_BIT_EXT;
        const PROTECTED_ACCESS_ONLY_EXT = vk::PIPELINE_CREATE_PROTECTED_ACCESS_ONLY_BIT_EXT;
    }

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

struct PipelineCache<'a> {
    inner: NonZeroU64,
    device: &'a Device
}

impl PipelineCache<'_> {
    #[inline]
    pub fn id (&self) -> u64 {
        return self.inner.get()
    }
}

impl Drop for PipelineCache<'_> {
    #[inline]
    fn drop(&mut self) {
        (Entry::get().destroy_pipeline_cache)(self.device.id(), self.id(), core::ptr::null())
    }
}

struct PipelineLayout<'a> {
    inner: NonZeroU64,
    device: &'a Device
}

impl PipelineLayout<'_> {
    #[inline]
    pub fn id (&self) -> u64 {
        return self.inner.get()
    }
}

impl Drop for PipelineLayout<'_> {
    #[inline]
    fn drop(&mut self) {
        (Entry::get().destroy_pipeline_layout)(self.device.id(), self.id(), core::ptr::null())
    }
}
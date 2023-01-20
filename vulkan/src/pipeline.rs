use std::{num::NonZeroU64, ptr::{addr_of, addr_of_mut}, ffi::CStr};
use crate::{shader::{LayoutCreateFlags, ShaderStages, Shader}, Entry, Result, device::{Device, DeviceRef}, utils::usize_to_u32, descriptor::{DescriptorType, DescriptorPool, DescriptorPoolFlags, DescriptorSets}};
use proc::cstr;

const DEFAULT_ENTRY: &CStr = cstr!("main");

pub struct ComputeBuilder<'a, D> {
    pipe_flags: PipelineFlags,
    pipe_layout_flags: PipelineLayoutFlags,
    cache_flags: Option<PipelineCacheFlags>,
    layout_flags: LayoutCreateFlags,
    bindings: Vec<vk::DescriptorSetLayoutBinding>,
    pool_sizes: Vec<vk::DescriptorPoolSize>,
    device: D,
    entry: &'a CStr
}

impl<'a, D: Clone + DeviceRef> ComputeBuilder<'a, D> {
    #[inline]
    pub fn new (device: D) -> Self {
        return Self {
            pipe_flags: PipelineFlags::empty(),
            pipe_layout_flags: PipelineLayoutFlags::empty(),
            layout_flags: LayoutCreateFlags::empty(),
            cache_flags: None,
            bindings: Vec::new(),
            pool_sizes: Vec::new(),
            entry: DEFAULT_ENTRY,
            device,
        }
    }

    #[inline]
    pub fn entry (mut self, entry: &'a CStr) -> Self {
        self.entry = entry;
        self
    }

    #[inline]
    pub fn binding (mut self, ty: DescriptorType, len: u32) -> Self {
        let mut done = false;
        for size in self.pool_sizes.iter_mut() {
            if size.typ == ty as vk::DescriptorType {
                size.descriptorCount += 1;
                done = true;
                break;
            }
        };

        if !done {
            self.pool_sizes.push(vk::DescriptorPoolSize { typ: ty as vk::DescriptorType, descriptorCount: 1 });
        }

        self.bindings.push(vk::DescriptorSetLayoutBinding {
            binding: usize_to_u32(self.bindings.len()),
            descriptorType: ty as vk::DescriptorType,
            descriptorCount: len,
            stageFlags: vk::SHADER_STAGE_COMPUTE_BIT,
            pImmutableSamplers: core::ptr::null(),
        });

        self
    }
    
    #[inline]
    pub fn flags (mut self, flags: PipelineFlags) -> Self {
        self.pipe_flags = flags;
        self
    }
    
    #[inline]
    pub fn layout_flags (mut self, flags: PipelineLayoutFlags) -> Self {
        self.pipe_layout_flags = flags;
        self
    }

    #[inline]
    pub fn cache_flags (mut self, flags: PipelineCacheFlags) -> Self {
        self.cache_flags = Some(flags);
        self
    }

    pub fn build (mut self, words: &[u32]) -> Result<Pipeline<D>> {
        let entry = Entry::get();
        let shader = self.build_shader(words)?;

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
                (entry.create_pipeline_cache)(self.device.id(), addr_of!(cache_info), core::ptr::null(), addr_of_mut!(my_cache))
            }
            if let Some(my_cache) = NonZeroU64::new(my_cache) {
                cache = Some(PipelineCache { inner: my_cache, device: self.device.clone() });
            } else {
                return Err(vk::ERROR_UNKNOWN.into())
            }
        }

        // Create pipeline layout
        let mut my_layout = 0;
        let layout_info = vk::PipelineLayoutCreateInfo {
            sType: vk::STRUCTURE_TYPE_PIPELINE_LAYOUT_CREATE_INFO,
            pNext: core::ptr::null(),
            flags: self.pipe_layout_flags.bits(),
            setLayoutCount: 1,
            pSetLayouts: &shader.layout(),
            pushConstantRangeCount: 0,
            pPushConstantRanges: core::ptr::null(),
        };
        tri! { (entry.create_pipeline_layout)(self.device.id(), addr_of!(layout_info), core::ptr::null(), addr_of_mut!(my_layout)) }
        let layout;
        if let Some(my_layout) = NonZeroU64::new(my_layout) {
            layout = my_layout;
        } else {
            return Err(vk::ERROR_UNKNOWN.into())
        }

        // Create pipeline (TODO FIX BUG)
        let mut pipeline = 0;
        let info = vk::ComputePipelineCreateInfo {
            sType: vk::STRUCTURE_TYPE_COMPUTE_PIPELINE_CREATE_INFO,
            pNext: core::ptr::null(),
            //flags: vk::PIPELINE_CREATE_DISABLE_OPTIMIZATION_BIT, 
            flags: self.pipe_flags.bits(),
            stage: vk::PipelineShaderStageCreateInfo {
                sType: vk::STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO,
                pNext: core::ptr::null(),
                flags: 0,
                stage: vk::SHADER_STAGE_COMPUTE_BIT,
                module: shader.module(),
                pName: self.entry.as_ptr(),
                //pName: b"compute_personal_event\0".as_ptr().cast(),
                pSpecializationInfo: core::ptr::null(),
            },
            layout: layout.get(),
            basePipelineHandle: vk::NULL_HANDLE,
            basePipelineIndex: 0,
        };

        match (entry.create_compute_pipelines)(
            self.device.id(),
            cache.as_ref().map_or(vk::NULL_HANDLE, PipelineCache::id),
            1,
            addr_of!(info),
            core::ptr::null(),
            addr_of_mut!(pipeline)
        ) {
            vk::SUCCESS => {},
            e => {
                (Entry::get().destroy_pipeline_layout)(self.device.id(), layout.get(), core::ptr::null());
                return Err(e.into())
            }
        }

        if let Some(inner) = NonZeroU64::new(pipeline) {
            let pool = match self.build_descriptor_pool() {
                Ok(x) => x,
                Err(e) => {
                    (Entry::get().destroy_pipeline)(self.device.id(), inner.get(), core::ptr::null());
                    return Err(e);
                }
            };

            let sets = DescriptorSets::new(pool, core::slice::from_ref(&shader))?;
            return Ok(Pipeline { inner, layout, sets })
        }

        (Entry::get().destroy_pipeline_layout)(self.device.id(), layout.get(), core::ptr::null());
        return Err(vk::ERROR_UNKNOWN.into())
    }

    fn build_shader (&mut self, words: &[u32]) -> Result<Shader<D>> {
        let builder = crate::shader::Builder {
            bindings: core::mem::take(&mut self.bindings),
            flags: self.layout_flags,
            stage: ShaderStages::COMPUTE,
            device: self.device.clone(),
            entry: self.entry,
        };

        return builder.build(words);
    }

    fn build_descriptor_pool (&mut self) -> Result<DescriptorPool<D>> {
        let builder = crate::descriptor::Builder {
            flags: DescriptorPoolFlags::empty(),
            capacity: 1,
            pool_sizes: core::mem::take(&mut self.pool_sizes),
            device: self.device.clone(),
        };

        return builder.build()
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Pipeline<D: DeviceRef> {
    inner: NonZeroU64,
    layout: NonZeroU64,
    sets: DescriptorSets<D>
}

impl<D: DeviceRef> Pipeline<D> {
    #[inline]
    pub fn id (&self) -> u64 {
        return self.inner.get()
    }

    #[inline]
    pub fn layout (&self) -> u64 {
        return self.layout.get()
    }

    #[inline]
    pub fn device (&self) -> &Device {
        return self.sets.device()
    }

    #[inline]
    pub fn sets (&self) -> &DescriptorSets<D> {
        return &self.sets
    }

    #[inline]
    pub fn sets_mut (&mut self) -> &mut DescriptorSets<D> {
        return &mut self.sets
    }

    #[inline]
    pub fn pool (&self) -> &DescriptorPool<D> {
        return self.sets.pool()
    }

    /*#[inline]
    pub fn compute<'b: 'a> (shader: &'b Shader<'a>) -> ComputeBuilder<'a, 'b> {
        return ComputeBuilder::new(shader)
    }*/
}

impl<D: DeviceRef> Drop for Pipeline<D> {
    #[inline]
    fn drop(&mut self) {
        (Entry::get().destroy_pipeline_layout)(self.device().id(), self.layout(), core::ptr::null());
        (Entry::get().destroy_pipeline)(self.device().id(), self.id(), core::ptr::null());
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
    pub struct PipelineShaderStages: vk::PipelineShaderStageCreateFlagBits {
        const ALLOW_VARYING_SUBGROUP_SIZE = vk::PIPELINE_SHADER_STAGE_CREATE_ALLOW_VARYING_SUBGROUP_SIZE_BIT;
        const REQUIRE_FULL_SUBGROUPS = vk::PIPELINE_SHADER_STAGE_CREATE_REQUIRE_FULL_SUBGROUPS_BIT;
        const ALLOW_VARYING_SUBGROUP_SIZE_EXT = vk::PIPELINE_SHADER_STAGE_CREATE_ALLOW_VARYING_SUBGROUP_SIZE_BIT_EXT;
        const REQUIRE_FULL_SUBGROUPS_EXT = vk::PIPELINE_SHADER_STAGE_CREATE_REQUIRE_FULL_SUBGROUPS_BIT_EXT;
    }

    #[repr(transparent)]
    pub struct PipelineStages: vk::PipelineStageFlagBits {
        /// Before subsequent commands are processed
        const TOP_OF_PIPE = vk::PIPELINE_STAGE_TOP_OF_PIPE_BIT;
        /// Draw/DispatchIndirect command fetch
        const DRAW_INDIRECT = vk::PIPELINE_STAGE_DRAW_INDIRECT_BIT;
        /// Vertex/index fetch
        const VERTEX_INPUT = vk::PIPELINE_STAGE_VERTEX_INPUT_BIT;
        /// Vertex shading
        const VERTEX_SHADER = vk::PIPELINE_STAGE_VERTEX_SHADER_BIT;
        /// Tessellation control shading
        const TESSELLATION_CONTROL_SHADER = vk::PIPELINE_STAGE_TESSELLATION_CONTROL_SHADER_BIT;
        /// Tessellation evaluation shading
        const TESSELLATION_EVALUATION_SHADER = vk::PIPELINE_STAGE_TESSELLATION_EVALUATION_SHADER_BIT;
        /// Geometry shading
        const GEOMETRY_SHADER = vk::PIPELINE_STAGE_GEOMETRY_SHADER_BIT;
        /// Fragment shading
        const FRAGMENT_SHADER = vk::PIPELINE_STAGE_FRAGMENT_SHADER_BIT;
        /// Early fragment (depth and stencil) tests
        const EARLY_FRAGMENT_TESTS = vk::PIPELINE_STAGE_EARLY_FRAGMENT_TESTS_BIT;
        /// Late fragment (depth and stencil) tests
        const LATE_FRAGMENT_TESTS = vk::PIPELINE_STAGE_LATE_FRAGMENT_TESTS_BIT;
        /// Color attachment writes
        const COLOR_ATTACHMENT_OUTPUT = vk::PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT;
        /// Compute shading
        const COMPUTE_SHADER = vk::PIPELINE_STAGE_COMPUTE_SHADER_BIT;
        /// Transfer/copy operations
        const TRANSFER = vk::PIPELINE_STAGE_TRANSFER_BIT;
        /// After previous commands have completed
        const BOTTOM_OF_PIPE = vk::PIPELINE_STAGE_BOTTOM_OF_PIPE_BIT;
        /// Indicates host (CPU) is a source/sink of the dependency
        const HOST = vk::PIPELINE_STAGE_HOST_BIT;
        /// All stages of the graphics pipeline
        const ALL_GRAPHICS = vk::PIPELINE_STAGE_ALL_GRAPHICS_BIT;
        /// All stages supported on the queue
        const ALL_COMMANDS = vk::PIPELINE_STAGE_ALL_COMMANDS_BIT;
        const NONE = vk::PIPELINE_STAGE_NONE;
        const TRANSFORM_FEEDBACK_EXT = vk::PIPELINE_STAGE_TRANSFORM_FEEDBACK_BIT_EXT;
        const CONDITIONAL_RENDERING_EXT = vk::PIPELINE_STAGE_CONDITIONAL_RENDERING_BIT_EXT; // A pipeline stage for conditional rendering predicate fetch
        const ACCELERATION_STRUCTURE_BUILD_KHR = vk::PIPELINE_STAGE_ACCELERATION_STRUCTURE_BUILD_BIT_KHR;
        const RAY_TRACING_SHADER_KHR = vk::PIPELINE_STAGE_RAY_TRACING_SHADER_BIT_KHR;
        const SHADING_RATE_IMAGE_NV = vk::PIPELINE_STAGE_SHADING_RATE_IMAGE_BIT_NV;
        const RAY_TRACING_SHADER_NV = vk::PIPELINE_STAGE_RAY_TRACING_SHADER_BIT_NV;
        const ACCELERATION_STRUCTURE_BUILD_NV = vk::PIPELINE_STAGE_ACCELERATION_STRUCTURE_BUILD_BIT_NV;
        const TASK_SHADER_NV = vk::PIPELINE_STAGE_TASK_SHADER_BIT_NV;
        const MESH_SHADER_NV = vk::PIPELINE_STAGE_MESH_SHADER_BIT_NV;
        const FRAGMENT_DENSITY_PROCESS_EXT = vk::PIPELINE_STAGE_FRAGMENT_DENSITY_PROCESS_BIT_EXT;
        const FRAGMENT_SHADING_RATE_ATTACHMENT_KHR = vk::PIPELINE_STAGE_FRAGMENT_SHADING_RATE_ATTACHMENT_BIT_KHR;
        const COMMAND_PREPROCESS_NV = vk::PIPELINE_STAGE_COMMAND_PREPROCESS_BIT_NV;
        const NONE_KHR = vk::PIPELINE_STAGE_NONE_KHR;
        const TASK_SHADER_EXT = vk::PIPELINE_STAGE_TASK_SHADER_BIT_EXT;
        const MESH_SHADER_EXT = vk::PIPELINE_STAGE_MESH_SHADER_BIT_EXT;
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct PipelineCache<D: DeviceRef> {
    inner: NonZeroU64,
    device: D
}

impl<D: DeviceRef> PipelineCache<D> {
    #[inline]
    pub fn id (&self) -> u64 {
        return self.inner.get()
    }
}

impl<D: DeviceRef> Drop for PipelineCache<D> {
    #[inline]
    fn drop(&mut self) {
        (Entry::get().destroy_pipeline_cache)(self.device.id(), self.id(), core::ptr::null())
    }
}
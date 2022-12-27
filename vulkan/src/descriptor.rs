use std::{num::NonZeroU64, ptr::{addr_of, addr_of_mut}};
use crate::{utils::usize_to_u32, Result, device::Device, Entry, shader::Shader};

pub struct Builder<'a> {
    pub(crate) flags: DescriptorPoolFlags,
    pub(crate) capacity: u32,
    pub(crate) pool_sizes: Vec<vk::DescriptorPoolSize>,
    pub(crate) device: &'a Device
}

impl<'a> Builder<'a> {
    #[inline]
    pub fn new (device: &'a Device, capacity: u32) -> Self {
        return Self {
            flags: DescriptorPoolFlags::empty(),
            capacity,
            pool_sizes: Vec::new(),
            device,
        }
    }

    #[inline]
    pub fn pool_size (mut self, ty: DescriptorType, count: u32) -> Self {
        self.pool_sizes.push(vk::DescriptorPoolSize { typ: ty as vk::DescriptorType, descriptorCount: count });
        self
    }

    #[inline]
    pub fn flags (mut self, flags: DescriptorPoolFlags) -> Self {
        self.flags = flags;
        self
    }

    #[inline]
    pub fn build (self) -> Result<DescriptorPool<'a>> {
        return DescriptorPool::new(self.device, self.capacity, self.flags, &self.pool_sizes)
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct DescriptorPool<'a> {
    inner: NonZeroU64,
    device: &'a Device
}

impl<'a> DescriptorPool<'a> {
    #[inline]
    pub fn builder (device: &'a Device, capacity: u32) -> Builder<'a> {
        return Builder::new(device, capacity)
    }

    pub fn new (device: &'a Device, capacity: u32, flags: DescriptorPoolFlags, pool_sizes: &[vk::DescriptorPoolSize]) -> Result<Self> {
        let info = vk::DescriptorPoolCreateInfo {
            sType: vk::STRUCTURE_TYPE_DESCRIPTOR_POOL_CREATE_INFO,
            pNext: core::ptr::null(),
            flags: flags.bits(),
            maxSets: capacity,
            poolSizeCount: usize_to_u32(pool_sizes.len()),
            pPoolSizes: pool_sizes.as_ptr(),
        };

        let mut inner = 0;
        tri! {
            (Entry::get().create_descriptor_pool)(
                device.id(),
                addr_of!(info),
                core::ptr::null(),
                addr_of_mut!(inner)
            )
        }
        
        if let Some(inner) = NonZeroU64::new(inner) {
            return Ok(Self { inner, device })
        }
        return Err(vk::ERROR_UNKNOWN.into())
    }

    #[inline]
    pub fn id (&self) -> u64 {
        return self.inner.get()
    }
}

impl Drop for DescriptorPool<'_> {
    #[inline]
    fn drop(&mut self) {
        (Entry::get().destroy_descriptor_pool)(self.device.id(), self.id(), core::ptr::null())
    }
}

pub struct DescriptorSet<'a, 'b> {
    pool: &'b DescriptorPool<'a>
}

impl<'a, 'b> DescriptorSet<'a, 'b> {
    #[inline]
    pub fn new (pool: &'b DescriptorPool<'a>, shaders: &[Shader<'a>]) {
        let info = vk::DescriptorSetAllocateInfo {
            sType: vk::STRUCTURE_TYPE_DESCRIPTOR_SET_ALLOCATE_INFO,
            pNext: core::ptr::null(),
            descriptorPool: pool.id(),
            descriptorSetCount: usize_to_u32(shaders.len()),
            pSetLayouts: shaders.iter().map(Shader::layout).collect::<Vec<_>>().as_ptr(),
        };

        (Entry::get().allo)
    }
}

bitflags::bitflags! {
    #[repr(transparent)]
    pub struct DescriptorPoolFlags: vk::DescriptorPoolCreateFlags {
        /// Descriptor sets may be freed individually
        const FREE_DESCRIPTOR_SET = vk::DESCRIPTOR_POOL_CREATE_FREE_DESCRIPTOR_SET_BIT;
        const UPDATE_AFTER_BIND = vk::DESCRIPTOR_POOL_CREATE_UPDATE_AFTER_BIND_BIT;
        const UPDATE_AFTER_BIND_EXT = vk::DESCRIPTOR_POOL_CREATE_UPDATE_AFTER_BIND_BIT_EXT;
        const HOST_ONLY_VALVE = vk::DESCRIPTOR_POOL_CREATE_HOST_ONLY_BIT_VALVE;
        const HOST_ONLY_EXT = vk::DESCRIPTOR_POOL_CREATE_HOST_ONLY_BIT_EXT;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum DescriptorType {
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
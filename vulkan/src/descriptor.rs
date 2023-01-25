use std::{num::NonZeroU64, ptr::{addr_of, addr_of_mut}, ops::{Deref, DerefMut}};
use crate::{utils::usize_to_u32, Result, device::{Device}, Entry, shader::Shader, buffer::Buffer, alloc::DeviceAllocator, context::{ContextRef, Context}};

pub struct Builder<C> {
    pub(crate) flags: DescriptorPoolFlags,
    pub(crate) capacity: u32,
    pub(crate) pool_sizes: Vec<vk::DescriptorPoolSize>,
    pub(crate) context: C
}

impl<C: ContextRef> Builder<C> {
    #[inline]
    pub fn new (context: C, capacity: u32) -> Self {
        return Self {
            flags: DescriptorPoolFlags::empty(),
            capacity,
            pool_sizes: Vec::new(),
            context,
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
    pub fn build (self) -> Result<DescriptorPool<C>> {
        return DescriptorPool::new(self.context, self.capacity, self.flags, &self.pool_sizes)
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct DescriptorPool<C: ContextRef> {
    inner: NonZeroU64,
    context: C
}

impl<C: ContextRef> DescriptorPool<C> {
    #[inline]
    pub fn builder (context: C, capacity: u32) -> Builder<C> {
        return Builder::new(context, capacity)
    }

    pub fn new (context: C, capacity: u32, flags: DescriptorPoolFlags, pool_sizes: &[vk::DescriptorPoolSize]) -> Result<Self> {
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
                context.device().id(),
                addr_of!(info),
                core::ptr::null(),
                addr_of_mut!(inner)
            )
        }
        
        if let Some(inner) = NonZeroU64::new(inner) {
            return Ok(Self { inner, context })
        }
        return Err(vk::ERROR_UNKNOWN.into())
    }

    #[inline]
    pub fn id (&self) -> u64 {
        return self.inner.get()
    }

    #[inline]
    pub fn context (&self) -> &Context {
        return &self.context
    }

    #[inline]
    pub fn owned_context (&self) -> C where C: Clone {
        return self.context.clone()
    }
    
    #[inline]
    pub fn device (&self) -> &Device {
        return self.context.device()
    }
}

impl<C: ContextRef> Drop for DescriptorPool<C> {
    #[inline]
    fn drop(&mut self) {
        (Entry::get().destroy_descriptor_pool)(self.device().id(), self.id(), core::ptr::null())
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct DescriptorSets<C: ContextRef> {
    inner: Box<[DescriptorSet]>,
    pool: DescriptorPool<C>
}

impl<C: ContextRef> DescriptorSets<C> {
    pub fn new<U: ContextRef> (pool: DescriptorPool<C>, shaders: &[Shader<U>]) -> Result<Self> {
        let layouts = shaders.iter().map(Shader::layout).collect::<Vec<_>>();
        let info = vk::DescriptorSetAllocateInfo {
            sType: vk::STRUCTURE_TYPE_DESCRIPTOR_SET_ALLOCATE_INFO,
            pNext: core::ptr::null(),
            descriptorPool: pool.id(),
            descriptorSetCount: usize_to_u32(shaders.len()),
            pSetLayouts: layouts.as_ptr(),
        };

        let mut sets = Box::<[vk::DescriptorSet]>::new_uninit_slice(shaders.len());
        tri! {
            (Entry::get().allocate_descriptor_sets)(
                pool.context.device().id(),
                addr_of!(info),
                sets.as_mut_ptr().cast()
            )
        }

        let sets = unsafe { sets.assume_init() };
        if sets.iter().any(|x| *x == 0) {
            return Err(vk::ERROR_UNKNOWN.into())
        }

        let inner = unsafe { Box::from_raw(Box::into_raw(sets) as *mut [DescriptorSet]) };
        return Ok(Self { inner, pool })
    }

    #[inline]
    pub fn pool (&self) -> &DescriptorPool<C> {
        return &self.pool
    }

    #[inline]
    pub fn context (&self) -> &Context {
        return self.pool.context()
    }

    #[inline]
    pub fn owned_context (&self) -> C where C: Clone {
        return self.pool.owned_context()
    }
    
    #[inline]
    pub fn device (&self) -> &Device {
        return self.pool.device()
    }
}

impl<C: ContextRef> DescriptorSets<C> {
    pub fn update<'b> (&mut self, write: impl IntoIterator<Item = &'b WriteDescriptorSet>) {
        let write = write.into_iter()
            .zip(0u32..)
            .map(|(x, i)| {
                let mut x = x.get();
                x.dstBinding = i;
                return x
            })
            .collect::<Vec<_>>();

        (Entry::get().update_descriptor_sets)(
            self.device().id(),
            usize_to_u32(write.len()),
            write.as_ptr(),
            0, // todo
            core::ptr::null() // todo
        );
    }
}

impl<C: ContextRef> Deref for DescriptorSets<C> {
    type Target = [DescriptorSet];

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<C: ContextRef> DerefMut for DescriptorSets<C> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<C: ContextRef> Drop for DescriptorSets<C> {
    #[inline]
    fn drop(&mut self) {
        let result = (Entry::get().free_descriptor_sets)(
            self.pool.context.device().id(),
            self.pool.id(),
            usize_to_u32(self.inner.len()),
            self.inner.as_ptr().cast()
        );

        #[cfg(debug_assertions)]
        if result != vk::SUCCESS {
            eprintln!("error dropping descriptor sets: {result}")
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct DescriptorSet {
    id: vk::DescriptorSet
}

impl DescriptorSet {
    #[inline]
    pub fn id (&self) -> u64 {
        return self.id;
    }

    #[inline]
    pub fn write_descriptor<T, A: DeviceAllocator> (&self, buf: &Buffer<T, A>) -> WriteDescriptorSet {
        let inner = vk::WriteDescriptorSet {
            sType: vk::STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET,
            pNext: core::ptr::null(),
            dstSet: self.id(),
            dstBinding: 0, // will be set later
            dstArrayElement: 0,
            descriptorCount: 1,
            descriptorType: DescriptorType::StorageBuffer as i32,
            pImageInfo: core::ptr::null(),
            pBufferInfo: core::ptr::null(),
            pTexelBufferView: core::ptr::null(),
        };

        return WriteDescriptorSet {
            inner,
            buffer: Some(buf.descriptor())
        }
    }
}

pub struct WriteDescriptorSet {
    inner: vk::WriteDescriptorSet,
    buffer: Option<vk::DescriptorBufferInfo>
}

impl WriteDescriptorSet {
    #[inline]
    pub fn get (&self) -> vk::WriteDescriptorSet {
        let mut this = self.inner.clone();
        if let Some(ref buffer) = self.buffer {
            this.pBufferInfo = buffer;
        }
        return this
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
#[non_exhaustive]
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
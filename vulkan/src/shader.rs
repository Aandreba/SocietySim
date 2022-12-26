use std::{ptr::{addr_of, addr_of_mut}, num::NonZeroU64};
use crate::{Result, Entry, device::Device};

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Shader<'a> {
    inner: NonZeroU64,
    device: &'a Device
}

impl<'a> Shader<'a> {
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
}

impl Drop for Shader<'_> {
    #[inline]
    fn drop(&mut self) {
        (Entry::get().destroy_shader_module)(self.device.id(), self.inner.get(), core::ptr::null())
    }
}
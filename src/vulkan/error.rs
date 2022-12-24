#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Vulkan (super::vk::Result),
    #[error("{0}")]
    Library (#[from] libloading::Error)
}

impl From<super::vk::Result> for Error {
    #[inline]
    fn from(value: super::vk::Result) -> Self {
        return Error::Vulkan(value)
    }
}
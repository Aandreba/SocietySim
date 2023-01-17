#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{} ({0})", result_name(*.0).unwrap_or(""))]
    Vulkan (super::vk::Result),
    #[error("{0}")]
    Library (#[from] libloading::Error)
}

impl Error {
    pub fn name (&self) -> Option<&'static str> {
        return match self {
            Self::Vulkan(e) => result_name(*e),
            _ => None
        }
    }
}

impl From<super::vk::Result> for Error {
    #[inline]
    fn from(value: super::vk::Result) -> Self {
        return Error::Vulkan(value)
    }
}

#[inline]
pub fn result_name (e: vk::Result) -> Option<&'static str> {
    return match e {
        vk::SUCCESS => Some("SUCCESS"), // Command completed successfully
        vk::NOT_READY => Some("NOT_READY"), // A fence or query has not yet completed
        vk::TIMEOUT => Some("TIMEOUT"), // A wait operation has not completed in the specified time
        vk::EVENT_SET => Some("EVENT_SET"), // An event is signaled
        vk::EVENT_RESET => Some("EVENT_RESET"), // An event is unsignaled
        vk::INCOMPLETE => Some("INCOMPLETE"), // A return array was too small for the result
        vk::ERROR_OUT_OF_HOST_MEMORY => Some("ERROR_OUT_OF_HOST_MEMORY"), // A host memory allocation has failed
        vk::ERROR_OUT_OF_DEVICE_MEMORY => Some("ERROR_OUT_OF_DEVICE_MEMORY"), // A device memory allocation has failed
        vk::ERROR_INITIALIZATION_FAILED => Some("ERROR_INITIALIZATION_FAILED"), // Initialization of an object has failed
        vk::ERROR_DEVICE_LOST => Some("ERROR_DEVICE_LOST"), // The logical device has been lost. See <<devsandqueues-lost-device>>
        vk::ERROR_MEMORY_MAP_FAILED => Some("ERROR_MEMORY_MAP_FAILED"), // Mapping of a memory object has failed
        vk::ERROR_LAYER_NOT_PRESENT => Some("ERROR_LAYER_NOT_PRESENT"), // Layer specified does not exist
        vk::ERROR_EXTENSION_NOT_PRESENT => Some("ERROR_EXTENSION_NOT_PRESENT"), // Extension specified does not exist
        vk::ERROR_FEATURE_NOT_PRESENT => Some("ERROR_FEATURE_NOT_PRESENT"), // Requested feature is not available on this device
        vk::ERROR_INCOMPATIBLE_DRIVER => Some("ERROR_INCOMPATIBLE_DRIVER"), // Unable to find a Vulkan driver
        vk::ERROR_TOO_MANY_OBJECTS => Some("ERROR_TOO_MANY_OBJECTS"), // Too many objects of the type have already been created
        vk::ERROR_FORMAT_NOT_SUPPORTED => Some("ERROR_FORMAT_NOT_SUPPORTED"), // Requested format is not supported on this device
        vk::ERROR_FRAGMENTED_POOL => Some("ERROR_FRAGMENTED_POOL"), // A requested pool allocation has failed due to fragmentation of the pool's memory
        vk::ERROR_UNKNOWN => Some("ERROR_UNKNOWN"), // An unknown error has occurred, due to an implementation or application bug
        vk::ERROR_OUT_OF_POOL_MEMORY => Some("ERROR_OUT_OF_POOL_MEMORY"),
        vk::ERROR_INVALID_EXTERNAL_HANDLE => Some("ERROR_INVALID_EXTERNAL_HANDLE"),
        vk::ERROR_FRAGMENTATION => Some("ERROR_FRAGMENTATION"),
        vk::ERROR_INVALID_OPAQUE_CAPTURE_ADDRESS => Some("ERROR_INVALID_OPAQUE_CAPTURE_ADDRESS"),
        vk::PIPELINE_COMPILE_REQUIRED => Some("PIPELINE_COMPILE_REQUIRED"),
        vk::ERROR_SURFACE_LOST_KHR => Some("ERROR_SURFACE_LOST_KHR"),
        vk::ERROR_NATIVE_WINDOW_IN_USE_KHR => Some("ERROR_NATIVE_WINDOW_IN_USE_KHR"),
        vk::SUBOPTIMAL_KHR => Some("SUBOPTIMAL_KHR"),
        vk::ERROR_OUT_OF_DATE_KHR => Some("ERROR_OUT_OF_DATE_KHR"),
        vk::ERROR_INCOMPATIBLE_DISPLAY_KHR => Some("ERROR_INCOMPATIBLE_DISPLAY_KHR"),
        vk::ERROR_VALIDATION_FAILED_EXT => Some("ERROR_VALIDATION_FAILED_EXT"),
        vk::ERROR_INVALID_SHADER_NV => Some("ERROR_INVALID_SHADER_NV"),
        vk::ERROR_IMAGE_USAGE_NOT_SUPPORTED_KHR => Some("ERROR_IMAGE_USAGE_NOT_SUPPORTED_KHR"),
        vk::ERROR_VIDEO_PICTURE_LAYOUT_NOT_SUPPORTED_KHR => Some("ERROR_VIDEO_PICTURE_LAYOUT_NOT_SUPPORTED_KHR"),
        vk::ERROR_VIDEO_PROFILE_OPERATION_NOT_SUPPORTED_KHR => Some("ERROR_VIDEO_PROFILE_OPERATION_NOT_SUPPORTED_KHR"),
        vk::ERROR_VIDEO_PROFILE_FORMAT_NOT_SUPPORTED_KHR => Some("ERROR_VIDEO_PROFILE_FORMAT_NOT_SUPPORTED_KHR"),
        vk::ERROR_VIDEO_PROFILE_CODEC_NOT_SUPPORTED_KHR => Some("ERROR_VIDEO_PROFILE_CODEC_NOT_SUPPORTED_KHR"),
        vk::ERROR_VIDEO_STD_VERSION_NOT_SUPPORTED_KHR => Some("ERROR_VIDEO_STD_VERSION_NOT_SUPPORTED_KHR"),
        // Vulkan(vk::ERROR_OUT_OF_POOL_MEMORY_KHR) => Some("ERROR_OUT_OF_POOL_MEMORY_KHR"),
        // Vulkan(vk::ERROR_INVALID_EXTERNAL_HANDLE_KHR) => Some("ERROR_INVALID_EXTERNAL_HANDLE_KHR"),
        vk::ERROR_INVALID_DRM_FORMAT_MODIFIER_PLANE_LAYOUT_EXT => Some("ERROR_INVALID_DRM_FORMAT_MODIFIER_PLANE_LAYOUT_EXT"),
        // Vulkan(vk::ERROR_FRAGMENTATION_EXT) => Some("ERROR_FRAGMENTATION_EXT"),
        vk::ERROR_NOT_PERMITTED_EXT => Some("ERROR_NOT_PERMITTED_EXT"),
        // Vulkan(vk::ERROR_NOT_PERMITTED_KHR) => Some("ERROR_NOT_PERMITTED_KHR"),
        // Vulkan(vk::ERROR_INVALID_DEVICE_ADDRESS_EXT) => Some("ERROR_INVALID_DEVICE_ADDRESS_EXT"),
        // Vulkan(vk::ERROR_INVALID_OPAQUE_CAPTURE_ADDRESS_KHR) => Some("ERROR_INVALID_OPAQUE_CAPTURE_ADDRESS_KHR"),
        vk::THREAD_IDLE_KHR => Some("THREAD_IDLE_KHR"),
        vk::THREAD_DONE_KHR => Some("THREAD_DONE_KHR"),
        vk::OPERATION_DEFERRED_KHR => Some("OPERATION_DEFERRED_KHR"),
        vk::OPERATION_NOT_DEFERRED_KHR => Some("OPERATION_NOT_DEFERRED_KHR"),
        // Vulkan(vk::PIPELINE_COMPILE_REQUIRED_EXT) => Some("PIPELINE_COMPILE_REQUIRED_EXT"),
        // Vulkan(vk::ERROR_PIPELINE_COMPILE_REQUIRED_EXT) => Some("ERROR_PIPELINE_COMPILE_REQUIRED_EXT"),
        vk::ERROR_COMPRESSION_EXHAUSTED_EXT => Some("ERROR_COMPRESSION_EXHAUSTED_EXT"),
        _ => None
    }
}
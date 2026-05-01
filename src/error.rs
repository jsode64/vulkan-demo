use ash::vk;
use thiserror::Error;

pub type Result<T = ()> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    /// An `std::io` error.
    #[error("IO error: \"{0}\"")]
    IoError(std::io::Error),

    /// A GLFW error.
    #[error("GLFW gave error cose {0}")]
    GlfwError(i32),

    /// A Vulkan error.
    #[error("Vulkan function returned error code {0}")]
    VkError(vk::Result),

    /// No suitable physical device was found.
    #[error("Found no suitable physical device")]
    NoSuitablePhysicalDevice,

    /// Tried to allocate memory to an incompatible allocator.
    #[error("Tried to allocate memory to an incompatible allocator")]
    AllocatorIncompatible,

    /// Tried to allocate too much memory into an allocator.
    #[error("Tried to allocate too much memory into an allocator")]
    AllocatorFull,
}

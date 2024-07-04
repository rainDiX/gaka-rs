use std::{ffi::NulError, fmt};

use ash::vk;

pub enum ContextInitError {
    SystemError,
    VulkanError(vk::Result),
    StringError,
}

impl From<NulError> for ContextInitError {
    fn from(_: NulError) -> Self {
        ContextInitError::StringError
    }
}

impl From<ash::LoadingError> for ContextInitError {
    fn from(value: ash::LoadingError) -> Self {
        println!("{}", value);
        ContextInitError::SystemError
    }
}

impl From<vk::Result> for ContextInitError {
    fn from(e: vk::Result) -> Self {
        ContextInitError::VulkanError(e)
    }
}

impl fmt::Display for ContextInitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SystemError => {
                fmt::Display::fmt("There was an error loading from system vulkan libraries", f)
            }
            Self::StringError => fmt::Display::fmt(
                "There was an error converting a string to a null terminated CString",
                f,
            ),
            Self::VulkanError(e) => fmt::Display::fmt(e, f),
        }
    }
}

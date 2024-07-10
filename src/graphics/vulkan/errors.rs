/*
* SPDX-License-Identifier: MIT
*/

use std::{ffi::NulError, fmt};

use ash::vk;

pub enum VulkanError {
    SystemError,
    VulkanError(vk::Result),
    DeviceSelectionError,
    StringError,
}

impl From<NulError> for VulkanError {
    fn from(_: NulError) -> Self {
        VulkanError::StringError
    }
}

impl From<ash::LoadingError> for VulkanError {
    fn from(value: ash::LoadingError) -> Self {
        println!("{}", value);
        VulkanError::SystemError
    }
}

impl From<vk::Result> for VulkanError {
    fn from(e: vk::Result) -> Self {
        VulkanError::VulkanError(e)
    }
}

impl fmt::Display for VulkanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SystemError => {
                fmt::Display::fmt("There was an error loading from system vulkan libraries", f)
            }
            Self::StringError => fmt::Display::fmt(
                "There was an error converting a string to a null terminated CString",
                f,
            ),
            Self::DeviceSelectionError => fmt::Display::fmt(
                "Selected device not available",
                f),
            Self::VulkanError(e) => fmt::Display::fmt(e, f)
        }
    }
}

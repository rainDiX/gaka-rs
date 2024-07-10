/*
* SPDX-License-Identifier: MIT
*/

use ash::vk;

use super::errors;
use super::utils;

pub struct Device {
    logical_device: ash::Device,
    graphics_queue: vk::Queue,

    properties: vk::PhysicalDeviceProperties,
}

impl Device {
    pub fn new(
        instance: &ash::Instance,
        physical_device: &vk::PhysicalDevice,
    ) -> Result<Self, errors::VulkanError> {
        let device_properties =
            unsafe { instance.get_physical_device_properties(*physical_device) };
        //let device_features = unsafe { instance.get_physical_device_features(*physical_device) };

        let device_type = match device_properties.device_type {
            vk::PhysicalDeviceType::CPU => "Cpu",
            vk::PhysicalDeviceType::INTEGRATED_GPU => "Integrated GPU",
            vk::PhysicalDeviceType::DISCRETE_GPU => "Discrete GPU",
            vk::PhysicalDeviceType::VIRTUAL_GPU => "Virtual GPU",
            vk::PhysicalDeviceType::OTHER => "Other",
            _ => "Unknown Device Type",
        };

        let device_name = unsafe { utils::raw_to_string(&device_properties.device_name) };
        log::info!(
            "Using Device: {}, id: {}, type: {}",
            device_name,
            device_properties.device_id,
            device_type
        );

        log::info!(
            "Vulkan API Version: {}.{}.{}",
            vk::api_version_major(device_properties.api_version),
            vk::api_version_minor(device_properties.api_version),
            vk::api_version_patch(device_properties.api_version)
        );
        Ok(Device {
            logical_device: todo!(),
            graphics_queue: todo!(),
            properties: todo!(),
        })
    }
}

/*
* SPDX-License-Identifier: MIT
*/

use std::ffi;

use ash::vk;

use super::errors;
use super::utils;

pub struct Device {
    logical_device: ash::Device,
    graphics_queue: vk::Queue,
    present_queue: vk::Queue,
}

impl Device {
    pub fn new(
        instance: &ash::Instance,
        physical_device: &vk::PhysicalDevice,
        graphics_family_index: u32,
        present_family_index: u32,
        extensions: &[ffi::CString],
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

        // TODO customize queue creation
        let queue_priority = [1.0f32];

        let mut queue_create_infos = Vec::new();
        let mut queue_create_info = vk::DeviceQueueCreateInfo::default();
        queue_create_info = queue_create_info.queue_family_index(graphics_family_index);
        queue_create_info.queue_count = 1;
        queue_create_info = queue_create_info.queue_priorities(&queue_priority);
        queue_create_infos.push(queue_create_info);

        let mut dev_create_info = vk::DeviceCreateInfo::default();
        dev_create_info = dev_create_info.queue_create_infos(&queue_create_infos);
        let extensions_raw = utils::string_slice_to_raw_slice(extensions);
        dev_create_info = dev_create_info.enabled_extension_names(&extensions_raw);

        let logical_device = unsafe {
            instance.create_device(*physical_device, &dev_create_info, None)?
        };

        let graphics_queue = unsafe { logical_device.get_device_queue(graphics_family_index, 0) };
        let present_queue = unsafe { logical_device.get_device_queue(present_family_index, 0) };

        Ok(Device {
            logical_device,
            graphics_queue,
            present_queue,
        })
    }
}

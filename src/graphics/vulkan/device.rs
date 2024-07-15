/*
* SPDX-License-Identifier: MIT
*/

use std::ffi;
use std::rc::Rc;
use std::rc::Weak;

use ash::vk;

use super::context;
use super::errors;
use super::swapchain;
use super::utils;

pub struct VulkanDevice {
    context: Weak<context::VulkanContext>,
    physical_device: vk::PhysicalDevice,
    logical_device: ash::Device,
    graphics_queue: vk::Queue,
    present_queue: vk::Queue,
    graphics_family_index: u32,
    present_family_index: u32,
}

impl VulkanDevice {
    pub fn new(
        ctx: &Rc<context::VulkanContext>,
        physical_device: vk::PhysicalDevice,
        graphics_family_index: u32,
        present_family_index: u32,
        extensions: &[ffi::CString],
    ) -> Result<Self, errors::VulkanError> {
        let device_properties = unsafe {
            ctx.instance()
                .get_physical_device_properties(physical_device)
        };
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
        let queue_priorities = [1.0_f32];

        let mut queue_create_infos = Vec::new();
        let mut queue_create_info = vk::DeviceQueueCreateInfo::default();
        queue_create_info = queue_create_info.queue_family_index(graphics_family_index);
        queue_create_info.queue_count = queue_priorities.len() as u32;
        queue_create_info = queue_create_info.queue_priorities(&queue_priorities);
        queue_create_infos.push(queue_create_info);

        let mut dev_create_info = vk::DeviceCreateInfo::default();
        dev_create_info = dev_create_info.queue_create_infos(&queue_create_infos);
        dev_create_info.queue_create_info_count = queue_create_infos.len() as u32;

        let extensions_raw = utils::string_slice_to_raw_slice(extensions);
        dev_create_info = dev_create_info.enabled_extension_names(&extensions_raw);
        dev_create_info.enabled_extension_count = extensions.len() as u32;

        let logical_device = unsafe {
            ctx.instance()
                .create_device(physical_device, &dev_create_info, None)?
        };

        let graphics_queue = unsafe { logical_device.get_device_queue(graphics_family_index, 0) };
        let present_queue = unsafe { logical_device.get_device_queue(present_family_index, 0) };

        Ok(VulkanDevice {
            context: Rc::downgrade(ctx),
            physical_device,
            logical_device,
            graphics_queue,
            present_queue,
            graphics_family_index,
            present_family_index,
        })
    }

    pub(crate) fn query_swapchain_support(&self) -> swapchain::SwapChainSupportDetail {
        let ctx = self.context.upgrade().expect("Failed to uprgrade reference to vulkan context");
        unsafe {
            let capabilities = ctx
                .surface_fn()
                .get_physical_device_surface_capabilities(
                    self.physical_device,
                    *ctx.surface(),
                )
                .expect("Failed to query for surface capabilities.");
            let formats = ctx
                .surface_fn()
                .get_physical_device_surface_formats(self.physical_device, *ctx.surface())
                .expect("Failed to query for surface formats.");
            let present_modes = ctx
                .surface_fn()
                .get_physical_device_surface_present_modes(
                    self.physical_device,
                    *ctx.surface(),
                )
                .expect("Failed to query for surface present mode.");

            swapchain::SwapChainSupportDetail {
                capabilities,
                formats,
                present_modes,
            }
        }
    }

    pub fn logical_device(&self) -> &ash::Device {
        &self.logical_device
    }

    pub fn create_swapchain(self: &Rc<Self>, width: u32, height: u32) -> swapchain::VulkanSwapChain {
        let (image_sharing_mode, queue_family_indices) =
            if self.graphics_family_index != self.present_family_index {
                (
                    vk::SharingMode::CONCURRENT,
                    vec![self.graphics_family_index, self.present_family_index],
                )
            } else {
                (vk::SharingMode::EXCLUSIVE, vec![])
            };

        let ctx = self.context.upgrade().expect("Failed to uprgrade reference to vulkan context");
        swapchain::VulkanSwapChain::new(
            &ctx,
            self,
            width,
            height,
            image_sharing_mode,
            &queue_family_indices,
        )
    }

}

impl Drop for VulkanDevice {
    fn drop(&mut self) {
        unsafe { self.logical_device.destroy_device(None) };
    }
}

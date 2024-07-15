/*
* SPDX-License-Identifier: MIT
*/

use std::rc::Rc;

use ash::{khr, vk};

use super::{context, device};

pub struct VulkanSwapChain {
    swapchain_device: khr::swapchain::Device,
    swapchain: vk::SwapchainKHR,
    swapchain_images: Vec<vk::Image>,
    swapchain_format: vk::Format,
    swapchain_extent: vk::Extent2D,
}

impl VulkanSwapChain {
    pub fn new(
        context: Rc<context::VulkanContext>,
        device: &device::VulkanDevice,
        width: u32,
        height: u32,
        image_sharing_mode: vk::SharingMode,
        queue_family_indices: &[u32],
    ) -> Self {
        let swapchain_support = device.query_swapchain_support();
        let surface_format = swapchain_support.choose_swapchain_format();
        let present_mode = swapchain_support.choose_swapchain_present_mode();
        let extent = swapchain_support.choose_swapchain_extent(width, height);

        let image_count = swapchain_support.capabilities.min_image_count + 1;
        let image_count = if swapchain_support.capabilities.max_image_count > 0 {
            image_count.min(swapchain_support.capabilities.max_image_count)
        } else {
            image_count
        };

        let swapchain_create_info = vk::SwapchainCreateInfoKHR::default()
            .flags(vk::SwapchainCreateFlagsKHR::empty())
            .surface(*context.surface())
            .min_image_count(image_count)
            .image_color_space(surface_format.color_space)
            .image_format(surface_format.format)
            .image_extent(extent)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(image_sharing_mode)
            .queue_family_indices(&queue_family_indices)
            .pre_transform(swapchain_support.capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true)
            .old_swapchain(vk::SwapchainKHR::null())
            .image_array_layers(1);

        log::info!("Creating SwapChain");

        let swapchain_device =
            khr::swapchain::Device::new(context.instance(), device.logical_device());
        let swapchain = unsafe {
            swapchain_device
                .create_swapchain(&swapchain_create_info, None)
                .expect("Failed to create Swapchain.")
        };

        let swapchain_images = unsafe {
            swapchain_device
                .get_swapchain_images(swapchain)
                .expect("Failed to get Swapchain Images.")
        };

        Self {
            swapchain_device,
            swapchain,
            swapchain_format: surface_format.format,
            swapchain_extent: extent,
            swapchain_images,
        }
    }
}

impl Drop for VulkanSwapChain {
    fn drop(&mut self) {
        unsafe {
            self.swapchain_device.destroy_swapchain(self.swapchain, None);
        }
    }
}

pub(crate) struct SwapChainSupportDetail {
    pub capabilities: vk::SurfaceCapabilitiesKHR,
    pub formats: Vec<vk::SurfaceFormatKHR>,
    pub present_modes: Vec<vk::PresentModeKHR>,
}

impl SwapChainSupportDetail {
    fn choose_swapchain_format(&self) -> vk::SurfaceFormatKHR {
        for available_format in &self.formats {
            if available_format.format == vk::Format::B8G8R8A8_SRGB
                && available_format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
            {
                return available_format.clone();
            }
        }
        // return the first format from the list
        return self.formats.first().unwrap().clone();
    }

    fn choose_swapchain_present_mode(&self) -> vk::PresentModeKHR {
        for &available_present_mode in self.present_modes.iter() {
            if available_present_mode == vk::PresentModeKHR::MAILBOX {
                return available_present_mode;
            }
        }
        vk::PresentModeKHR::FIFO
    }

    fn choose_swapchain_extent(&self, width: u32, height: u32) -> vk::Extent2D {
        if self.capabilities.current_extent.width != u32::max_value() {
            self.capabilities.current_extent
        } else {
            vk::Extent2D {
                width: width
                    .max(self.capabilities.min_image_extent.width)
                    .min(self.capabilities.max_image_extent.width),
                height: height
                    .max(self.capabilities.min_image_extent.height)
                    .min(self.capabilities.max_image_extent.height),
            }
        }
    }
}
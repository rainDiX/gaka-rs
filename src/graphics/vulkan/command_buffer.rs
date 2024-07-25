/*
* SPDX-License-Identifier: MIT
*/

use std::rc::Rc;

use ash::vk;

use super::{device::VulkanDevice, errors::VulkanError, graphics_pipeline::VulkanGraphicsPipeline};

pub struct VulkanCommandBuffer {
    device: Rc<VulkanDevice>,
    command_pool: vk::CommandPool,
    command_buffer: vk::CommandBuffer,
}

impl VulkanCommandBuffer {
    pub(crate) fn new(device: Rc<VulkanDevice>, command_pool: vk::CommandPool, command_buffer: vk::CommandBuffer ) -> Self {
        Self {
            device,
            command_pool,
            command_buffer
        }
    }

    pub fn record(&self, pipneline: &VulkanGraphicsPipeline, image_index: u32) -> Result<(), VulkanError>{
        let begin_info = vk::CommandBufferBeginInfo::default();
        unsafe {
            self.device.logical_device().begin_command_buffer(self.command_buffer, &begin_info)?;
        }
        let render_pass_info = vk::RenderPassBeginInfo::default()
            .render_pass(pipneline.render_pass());
        // TODO
        Ok(())
    }
}

impl Drop for VulkanCommandBuffer {
    fn drop(&mut self) {
        let command_buffers = [self.command_buffer];
        unsafe {
            self.device.logical_device().free_command_buffers(self.command_pool, &command_buffers);
            self.device
                .logical_device()
                .destroy_command_pool(self.command_pool, None)
        }
    }
}

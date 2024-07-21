/*
* SPDX-License-Identifier: MIT
*/

use std::rc::Rc;

use ash::vk;

use super::device::VulkanDevice;

pub struct VulkanGraphicsPipeline {
    pub device: Rc<VulkanDevice>,
    pub layout: vk::PipelineLayout,
    pub pipeline: vk::Pipeline,
    pub render_pass: vk::RenderPass,
}

impl Drop for VulkanGraphicsPipeline {
    fn drop(&mut self) {
        unsafe {
            self.device
                .logical_device()
                .destroy_pipeline(self.pipeline, None);
            self.device
                .logical_device()
                .destroy_pipeline_layout(self.layout, None);
            self.device
                .logical_device()
                .destroy_render_pass(self.render_pass, None);
        }
    }
}

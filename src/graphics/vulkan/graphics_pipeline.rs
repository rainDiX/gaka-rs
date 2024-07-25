/*
* SPDX-License-Identifier: MIT
*/

use std::rc::Rc;

use ash::vk;

use super::device::VulkanDevice;

pub struct VulkanGraphicsPipeline {
    device: Rc<VulkanDevice>,
    layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,
    render_pass: vk::RenderPass,
}

impl VulkanGraphicsPipeline {
    pub(crate) fn new(
        device: Rc<VulkanDevice>,
        layout: vk::PipelineLayout,
        pipeline: vk::Pipeline,
        render_pass: vk::RenderPass,
    ) -> Self {
        Self {
            device,
            layout,
            pipeline,
            render_pass,
        }
    }

    pub(crate) fn render_pass(&self) -> vk::RenderPass {
        self.render_pass
    }
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

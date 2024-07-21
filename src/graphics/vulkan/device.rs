/*
* SPDX-License-Identifier: MIT
*/

use std::ffi;
use std::rc::Rc;

use ash::vk;

use super::context;
use super::context::VulkanContext;
use super::errors;
use super::errors::VulkanError;
use super::graphics_pipeline::VulkanGraphicsPipeline;
use super::swapchain;
use super::swapchain::VulkanSwapChain;
use super::utils;

pub struct VulkanDevice {
    context: Rc<context::VulkanContext>,
    phy: vk::PhysicalDevice,
    device: ash::Device,
    graphics_queue: vk::Queue,
    present_queue: vk::Queue,
    graphics_family_index: u32,
    present_family_index: u32,
}

impl VulkanDevice {
    pub fn new(
        context: Rc<context::VulkanContext>,
        physical_device: vk::PhysicalDevice,
        graphics_family_index: u32,
        present_family_index: u32,
        extensions: &[ffi::CString],
    ) -> Result<Self, errors::VulkanError> {
        let device_properties = unsafe {
            context.instance()
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
            context.instance()
                .create_device(physical_device, &dev_create_info, None)?
        };

        let graphics_queue = unsafe { logical_device.get_device_queue(graphics_family_index, 0) };
        let present_queue = unsafe { logical_device.get_device_queue(present_family_index, 0) };

        Ok(VulkanDevice {
            context: context.clone(),
            phy: physical_device,
            device: logical_device,
            graphics_queue,
            present_queue,
            graphics_family_index,
            present_family_index,
        })
    }

    pub fn context(&self) -> Rc<VulkanContext> {
        self.context.clone()
    }

    pub(crate) fn query_swapchain_support(&self) -> swapchain::SwapChainSupportDetail {
        unsafe {
            let capabilities = self.context
                .surface_fn()
                .get_physical_device_surface_capabilities(self.phy, *self.context.surface())
                .expect("Failed to query for surface capabilities.");
            let formats = self.context
                .surface_fn()
                .get_physical_device_surface_formats(self.phy, *self.context.surface())
                .expect("Failed to query for surface formats.");
            let present_modes = self.context
                .surface_fn()
                .get_physical_device_surface_present_modes(self.phy, *self.context.surface())
                .expect("Failed to query for surface present mode.");

            swapchain::SwapChainSupportDetail {
                capabilities,
                formats,
                present_modes,
            }
        }
    }

    pub fn logical_device(&self) -> &ash::Device {
        &self.device
    }

    pub fn create_swapchain(
        self: &Rc<Self>,
        width: u32,
        height: u32,
    ) -> swapchain::VulkanSwapChain {
        let (image_sharing_mode, queue_family_indices) =
            if self.graphics_family_index != self.present_family_index {
                (
                    vk::SharingMode::CONCURRENT,
                    vec![self.graphics_family_index, self.present_family_index],
                )
            } else {
                (vk::SharingMode::EXCLUSIVE, vec![])
            };

        swapchain::VulkanSwapChain::new(
            self.clone(),
            width,
            height,
            image_sharing_mode,
            &queue_family_indices,
        )
    }

    unsafe fn create_shader_module(&self, code: &[u8]) -> Result<vk::ShaderModule, vk::Result> {
        let create_info = {
            let mut create_info = vk::ShaderModuleCreateInfo::default();
            create_info.code_size = code.len();
            create_info.p_code = code.as_ptr() as *const u32;
            create_info
        };
        self.device.create_shader_module(&create_info, None)
    }

    unsafe fn create_render_pass(
        &self,
        image_format: vk::Format,
    ) -> Result<vk::RenderPass, vk::Result> {
        let color_attachments = [vk::AttachmentDescription::default()
            .format(image_format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)];
        let color_attachments_ref = [
            vk::AttachmentReference::default().layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
        ];

        let subpasses = [vk::SubpassDescription::default()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&color_attachments_ref)];

        let render_pass_info = vk::RenderPassCreateInfo::default()
            .attachments(&color_attachments)
            .subpasses(&subpasses);

        unsafe { self.device.create_render_pass(&render_pass_info, None) }
    }

    pub fn create_graphics_pipeline(
        self: &Rc<Self>,
        swapchain: VulkanSwapChain,
        stride: u32,
        vertex_shader: &[u8],
        vs_entrypoint: &ffi::CStr,
        fragment_shader: &[u8],
        fs_entrypoint: &ffi::CStr,
        specializations: Option<&Vec<(u32, u32)>>,
    ) -> Result<VulkanGraphicsPipeline, VulkanError> {
        let render_pass = unsafe { self.create_render_pass(swapchain.format)? };

        let vertex_shader_mod = unsafe { self.create_shader_module(vertex_shader)? };
        let fragment_shader_mod = unsafe { self.create_shader_module(fragment_shader)? };

        let entries;
        let buffer;

        let specialization_info = if let Some(s) = specializations {
            (entries, buffer) = utils::create_specialization_entries(s);
            vk::SpecializationInfo::default()
                .map_entries(&entries)
                .data(&buffer)
        } else {
            vk::SpecializationInfo::default()
        };

        let vertex_stage_info = vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::VERTEX)
            .module(vertex_shader_mod)
            .name(vs_entrypoint)
            .specialization_info(&specialization_info);

        let fragment_stage_info = vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::FRAGMENT)
            .module(vertex_shader_mod)
            .name(fs_entrypoint)
            .specialization_info(&specialization_info);

        let shader_stages = [vertex_stage_info, fragment_stage_info];

        // make viewport and scissors dynamic NOTE: this should have no overhead in most vulkan implementations
        let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic_states_info =
            vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);
        let vertex_bindings = [vk::VertexInputBindingDescription::default()
            .binding(0)
            .stride(stride)
            .input_rate(vk::VertexInputRate::VERTEX)];

        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::default()
            .vertex_binding_descriptions(&vertex_bindings);
        // TODO return here to add missing vertex attribute description

        // TODO : make this also configurable
        let input_asm = vk::PipelineInputAssemblyStateCreateInfo::default()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false); // TODO allows to break strip with 0xFFFF

        let viewport_state = vk::PipelineViewportStateCreateInfo::default()
            .viewport_count(1)
            .scissor_count(1);

        let rasterizer_state = vk::PipelineRasterizationStateCreateInfo::default()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .cull_mode(vk::CullModeFlags::BACK)
            .front_face(vk::FrontFace::CLOCKWISE)
            .depth_bias_enable(false); // necessary for shadow mapping

        let multisample_state = vk::PipelineMultisampleStateCreateInfo::default()
            .sample_shading_enable(false) //TODO: multisampling is disabled for now
            .rasterization_samples(vk::SampleCountFlags::TYPE_1);
        // TODO: depth and stencil buffer

        // color blending
        let color_blend_attachments = [vk::PipelineColorBlendAttachmentState::default()
            .blend_enable(false)
            .color_write_mask(vk::ColorComponentFlags::RGBA)
            .src_color_blend_factor(vk::BlendFactor::ONE)
            .dst_color_blend_factor(vk::BlendFactor::ZERO)
            .color_blend_op(vk::BlendOp::ADD)
            .src_alpha_blend_factor(vk::BlendFactor::ONE)
            .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
            .alpha_blend_op(vk::BlendOp::ADD)];

        let color_blend_state = vk::PipelineColorBlendStateCreateInfo::default()
            .logic_op_enable(false)
            .logic_op(vk::LogicOp::COPY)
            .attachments(&color_blend_attachments)
            .blend_constants([0.0, 0.0, 0.0, 0.0]);

        let pipeline_layout_info = vk::PipelineLayoutCreateInfo::default();

        let pipeline_layout = unsafe {
            self.device
                .create_pipeline_layout(&pipeline_layout_info, None)?
        };

        let pipeline_infos = [vk::GraphicsPipelineCreateInfo::default()
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input_info)
            .input_assembly_state(&input_asm)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterizer_state)
            .multisample_state(&multisample_state)
            .color_blend_state(&color_blend_state)
            .dynamic_state(&dynamic_states_info)
            .layout(pipeline_layout)
            .render_pass(render_pass)
            .subpass(0)];

        let pipeline = unsafe {
            self.device
                .create_graphics_pipelines(vk::PipelineCache::null(), &pipeline_infos, None)
                .expect("failed to create graphics pipeline")
        };

        unsafe {
            self.device.destroy_shader_module(vertex_shader_mod, None);
            self.device.destroy_shader_module(fragment_shader_mod, None);
        }

        Ok(VulkanGraphicsPipeline {
            device: self.clone(),
            layout: pipeline_layout,
            pipeline: pipeline[0],
            render_pass: render_pass,
        })
    }
}

impl Drop for VulkanDevice {
    fn drop(&mut self) {
        unsafe { self.device.destroy_device(None) };
    }
}

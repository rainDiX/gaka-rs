/*
* SPDX-License-Identifier: MIT
*/

use std::ffi;
use std::ffi::CStr;
use std::ffi::CString;
use std::ptr;

use ash::ext;
use ash::vk;

extern "system" fn vulkan_debug_callback(
    severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    type_: vk::DebugUtilsMessageTypeFlagsEXT,
    data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _: *mut ffi::c_void,
) -> vk::Bool32 {
    let data = unsafe { *data };
    let message = unsafe { CStr::from_ptr(data.p_message) }.to_string_lossy();
    let message_type = match type_ {
        vk::DebugUtilsMessageTypeFlagsEXT::GENERAL => "General",
        vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE => "Performance",
        vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION => "Validation",
        _ => "Unknown",
    };

    match severity {
        vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => {
            log::error!("({}) {}", message_type, message)
        }
        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => {
            log::warn!("({}) {}", message_type, message)
        }
        vk::DebugUtilsMessageSeverityFlagsEXT::INFO => {
            log::debug!("({}) {}", message_type, message)
        }
        _ => log::trace!("({}) {}", message_type, message),
    }

    vk::FALSE
}

pub(crate) unsafe fn raw_to_string(raw_string_array: &[ffi::c_char]) -> String {
    let raw_str = CStr::from_ptr(raw_string_array.as_ptr());
    raw_str
        .to_str()
        .unwrap_or("Failed decode string")
        .to_owned()
}

pub(crate) fn string_slice_to_raw_slice(vector: &[CString]) -> Vec<*const ffi::c_char> {
    vector
        .iter()
        .map(|raw_string| raw_string.as_ptr())
        .collect()
}

pub(crate) unsafe fn layer_in_layer_properties(
    layer_list: &[vk::LayerProperties],
    layer_name: &str,
) -> bool {
    let namebytes: &[i8] = std::mem::transmute(layer_name);
    let layer = layer_list
        .iter()
        .find(|l| l.layer_name.starts_with(namebytes));
    layer.is_some()
}

pub(crate) fn create_debug_messenger(
    entry: &ash::Entry,
    instance: &ash::Instance,
) -> Result<(ext::debug_utils::Instance, vk::DebugUtilsMessengerEXT), vk::Result> {
    let create_info = vk::DebugUtilsMessengerCreateInfoEXT {
        s_type: vk::StructureType::DEBUG_UTILS_MESSENGER_CREATE_INFO_EXT,
        p_next: ptr::null(),
        flags: vk::DebugUtilsMessengerCreateFlagsEXT::empty(),
        message_severity: vk::DebugUtilsMessageSeverityFlagsEXT::WARNING |
            // vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE |
            // vk::DebugUtilsMessageSeverityFlagsEXT::INFO |
            vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
        message_type: vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
            | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
            | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
        pfn_user_callback: Some(vulkan_debug_callback),
        p_user_data: ptr::null_mut(),
        _marker: std::marker::PhantomData,
    };
    let dbg_instance = ext::debug_utils::Instance::new(entry, instance);
    unsafe {
        match dbg_instance.create_debug_utils_messenger(&create_info, None) {
            Ok(dbg_messenger) => Ok((dbg_instance, dbg_messenger)),
            Err(e) => Err(e),
        }
    }
}

pub(crate) fn is_physical_device_suitable(
    instance: &ash::Instance,
    physical_device: &vk::PhysicalDevice,
    required_ext: &[CString],
) -> bool {
    let extensions = unsafe {
        instance
            .enumerate_device_extension_properties(*physical_device)
            .unwrap_or_default()
            .iter()
            .map(|ext| CStr::from_ptr(ext.extension_name.as_ptr()).to_owned())
            .collect::<Vec<CString>>()
    };
    required_ext
        .iter()
        .all(|required| extensions.contains(required))
}

pub(crate) fn create_image_views(
    image_format: vk::Format,
    device: &ash::Device,
    images: &Vec<vk::Image>,
) -> Vec<vk::ImageView> {
    let mut swapchain_imageviews = vec![];

    for &image in images.iter() {
        let imageview_create_info = vk::ImageViewCreateInfo::default()
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(image_format)
            .components(vk::ComponentMapping {
                r: vk::ComponentSwizzle::IDENTITY,
                g: vk::ComponentSwizzle::IDENTITY,
                b: vk::ComponentSwizzle::IDENTITY,
                a: vk::ComponentSwizzle::IDENTITY,
            })
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            })
            .image(image);

        let imageview = unsafe {
            device
                .create_image_view(&imageview_create_info, None)
                .expect("Failed to create Image View!")
        };
        swapchain_imageviews.push(imageview);
    }

    swapchain_imageviews
}

pub(crate) fn create_specialization_entries(
    specialization_map: &[(u32, u32)],
) -> (Vec<vk::SpecializationMapEntry>, Vec<u8>) {
    let mut entries = Vec::new();
    let mut buffer = Vec::<u8>::new();
    let mut offset = 0;
    let size = std::mem::size_of::<u32>();
    for (id, value) in specialization_map.iter() {
        entries.push(
            vk::SpecializationMapEntry::default()
                .constant_id(*id)
                .size(size)
                .offset(offset),
        );
        unsafe {
            let bytes = std::mem::transmute::<u32, [u8; 4]>(*value);
            buffer.extend_from_slice(&bytes);
        }
        offset += 1;
    }
    (entries, buffer)
}

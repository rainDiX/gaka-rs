use std::ffi;
use std::ffi::CStr;
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

    if severity >= vk::DebugUtilsMessageSeverityFlagsEXT::ERROR {
        log::error!("({}) {}", message_type, message);
    } else if severity >= vk::DebugUtilsMessageSeverityFlagsEXT::WARNING {
        log::warn!("({}) {}", message_type, message);
    } else if severity >= vk::DebugUtilsMessageSeverityFlagsEXT::INFO {
        log::debug!("({}) {}", message_type, message);
    } else {
        log::trace!("({}) {}", message_type, message);
    }

    vk::FALSE
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
    ) -> bool {

        let device_queue_families =
            unsafe { instance.get_physical_device_queue_family_properties(*physical_device) };

        device_queue_families.iter().any(
            | queue_family | queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS)
        )
    }

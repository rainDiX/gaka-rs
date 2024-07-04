/*
* SPDX-License-Identifier: MIT
*/

use super::errors;
use super::utils;
use crate::constants;

use std::ffi::{self, CString};

use ash::ext;
use ash::vk;
use raw_window_handle::RawDisplayHandle;

pub struct VulkanContext {
    pub instance: ash::Instance,
    pub physical_devices: Vec<vk::PhysicalDevice>,

    dbg_instance: Option<ext::debug_utils::Instance>,
    debug_messenger: Option<vk::DebugUtilsMessengerEXT>,
}

impl VulkanContext {
    /// Returns a Vulkan Context for the Application
    ///
    /// # Arguments
    ///
    /// * `app` - A string slice that holds the name of the application
    /// * `version` - A 32 bit unsigned for the app version
    ///
    pub fn new(
        app: &str,
        version: u32,
        window_handle: &RawDisplayHandle,
        extensions: Option<&[&str]>,
        layers: Option<&[&str]>,
    ) -> Result<VulkanContext, errors::ContextInitError> {
        let app_name = CString::new(app)?;
        let engine_name = CString::new(constants::ENGINE_NAME)?;

        let entry = unsafe { ash::Entry::load()? };

        let appinfo = vk::ApplicationInfo::default()
            .application_name(app_name.as_c_str())
            .application_version(version)
            .engine_name(&engine_name.as_c_str())
            .engine_version(constants::ENGINE_VERSION)
            .api_version(vk::make_api_version(0, 1, 0, 0));

        let create_flags = if cfg!(any(target_os = "macos", target_os = "ios")) {
            vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR
        } else {
            vk::InstanceCreateFlags::default()
        };

        let mut layer_names = Vec::new();

        let surface_extensions = ash_window::enumerate_required_extensions(*window_handle)?;

        let mut extension_names = Vec::new();
        unsafe {
            let available_layers = entry.enumerate_instance_layer_properties()?;

            if layers.is_some() {
                layers.unwrap().iter().for_each(|l| {
                    if utils::layer_in_layer_properties(&available_layers, *l) {
                        layer_names.push(CString::new(*l).unwrap_or_default())
                    }
                });
            }

            #[cfg(debug_assertions)]
            layer_names.push(CString::from_vec_with_nul_unchecked(
                b"VK_LAYER_KHRONOS_validation\0".to_vec(),
            ));

            if extensions.is_some() {
                extensions
                    .unwrap()
                    .iter()
                    .for_each(|l| extension_names.push(CString::new(*l).unwrap_or_default()));
            }
        };

        let layer_names_raw: Vec<*const ffi::c_char> = layer_names
            .iter()
            .map(|raw_name| raw_name.as_ptr())
            .collect();

        let extension_names_raw = {
            let mut names_raw: Vec<*const ffi::c_char> = extension_names
                .iter()
                .map(|raw_name| raw_name.as_ptr())
                .collect();
            surface_extensions
                .iter()
                .for_each(|ext| names_raw.push(*ext));
            names_raw
        };

        let create_info = vk::InstanceCreateInfo::default()
            .application_info(&appinfo)
            .enabled_layer_names(&layer_names_raw)
            .enabled_extension_names(&extension_names_raw)
            .flags(create_flags);

        unsafe {
            let instance: ash::Instance = entry.create_instance(&create_info, None)?;

            #[cfg(debug_assertions)]
            let (dbg_instance, dbg_messenger) = utils::create_debug_messenger(&entry, &instance)?;

            let devices = instance.enumerate_physical_devices()?;

            #[cfg(debug_assertions)]
            return Ok(Self {
                instance: instance,
                physical_devices: devices,

                dbg_instance: Some(dbg_instance),
                debug_messenger: Some(dbg_messenger),
            });

            #[cfg(not(debug_assertions))]
            return Ok(Self {
                instance: instance,
                physical_devices: devices,

                dbg_instance: None,
                debug_messenger: None,
            });
        }
    }

    fn create_surface() {

    }

    fn get_device() {

    }
}

impl Drop for VulkanContext {
    fn drop(&mut self) {
        unsafe {
            match self.debug_messenger {
                Some(debug_messenger) => {
                    self.dbg_instance
                        .as_ref()
                        .unwrap()
                        .destroy_debug_utils_messenger(debug_messenger, None);
                }
                None => {}
            }
            self.instance.destroy_instance(None);
        }
    }
}

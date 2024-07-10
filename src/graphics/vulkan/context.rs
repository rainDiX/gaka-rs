/*
* SPDX-License-Identifier: MIT
*/

use super::device;
use super::errors;
use super::utils;
use crate::constants;

use std::ffi::{self, CString};

use ash::{ext, khr, vk};
use raw_window_handle::{RawDisplayHandle, RawWindowHandle};

pub struct VulkanPhysicalDevice {
    pub phy: vk::PhysicalDevice,
    pub graphics_family_index: Option<u32>,
    pub present_family_index: Option<u32>,
    pub suitable: bool, // support required extensions
}

impl VulkanPhysicalDevice {
    fn new(
        instance: &ash::Instance,
        phy: vk::PhysicalDevice,
        surface: &vk::SurfaceKHR,
        surface_fn: &khr::surface::Instance,
        required_ext: &[CString],
    ) -> VulkanPhysicalDevice {
        let mut graphics_queue_index = None;
        let mut present_queue_index = None;
        let queue_families = unsafe { instance.get_physical_device_queue_family_properties(phy) };

        let mut index = 0;
        for queue_family in queue_families.iter() {
            if queue_family.queue_count > 0
                && queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS)
            {
                graphics_queue_index = Some(index);
            };

            let present_support = unsafe {
                surface_fn
                    .get_physical_device_surface_support(phy, index, *surface)
                    .unwrap_or(false)
            };
            if queue_family.queue_count > 0 && present_support {
                present_queue_index = Some(index);
            }

            if graphics_queue_index.is_some() && present_queue_index.is_some() {
                break;
            }
            index += 1;
        }

        VulkanPhysicalDevice {
            phy,
            graphics_family_index: graphics_queue_index,
            present_family_index: present_queue_index,
            suitable: utils::is_physical_device_suitable(&instance, &phy, &required_ext),
        }
    }

    fn is_complete(&self) -> bool {
        self.graphics_family_index.is_some() && self.present_family_index.is_some()
    }
}

pub struct VulkanContext {
    pub entry: ash::Entry,
    pub physical_devices: Vec<VulkanPhysicalDevice>,

    instance: ash::Instance,
    surface_fn: khr::surface::Instance,
    surface: vk::SurfaceKHR,

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
        display_handle: &RawDisplayHandle,
        window_handle: &RawWindowHandle,
        extensions: Option<&[&str]>,
        layers: Option<&[&str]>,
    ) -> Result<VulkanContext, errors::VulkanError> {
        let app_name = CString::new(app)?;
        let engine_name = CString::new(constants::ENGINE_NAME)?;

        let entry: ash::Entry = unsafe { ash::Entry::load()? };

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

        let surface_extensions = ash_window::enumerate_required_extensions(*display_handle)?;

        let mut extension_names;
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
            layer_names.push(c"VK_LAYER_KHRONOS_validation".to_owned());

            extension_names = surface_extensions
                .iter()
                .map(|s| ffi::CStr::from_ptr(*s).to_owned())
                .collect::<Vec<CString>>();

            if extensions.is_some() {
                extensions
                    .unwrap()
                    .iter()
                    .for_each(|l| extension_names.push(CString::new(*l).unwrap_or_default()));
            }
            extension_names.push(c"VK_KHR_swapchain".to_owned());
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
            log::info!("Creating Vulkan instance");
            let instance: ash::Instance = entry.create_instance(&create_info, None)?;

            let surface_fn = ash::khr::surface::Instance::new(&entry, &instance);

            log::info!("Creating Vulkan surface");
            let surface = ash_window::create_surface(
                &entry,
                &instance,
                *display_handle,
                *window_handle,
                None,
            )?;

            #[cfg(debug_assertions)]
            let (dbg_instance, dbg_messenger) = utils::create_debug_messenger(&entry, &instance)?;

            let devices = instance
                .enumerate_physical_devices()?
                .iter()
                .map(|phy| {
                    VulkanPhysicalDevice::new(
                        &instance,
                        *phy,
                        &surface,
                        &surface_fn,
                        &extension_names,
                    )
                })
                .collect::<Vec<VulkanPhysicalDevice>>();

            #[cfg(debug_assertions)]
            return Ok(Self {
                entry: entry,
                physical_devices: devices,

                instance: instance,
                surface_fn: surface_fn,
                surface: surface,

                dbg_instance: Some(dbg_instance),
                debug_messenger: Some(dbg_messenger),
            });

            #[cfg(not(debug_assertions))]
            return Ok(Self {
                entry: entry,
                physical_devices: devices,

                instance: instance,
                surface_fn: surface_fn,
                surface: surface,

                dbg_instance: Some(dbg_instance),
                debug_messenger: Some(dbg_messenger),
            });
        }
    }

    pub fn create_graphic_device_default(&mut self) -> Result<device::Device, errors::VulkanError> {
        let mut selected_device = None;
        let mut selected_device_type = vk::PhysicalDeviceType::OTHER;

        let mut index = 0;

        for dev in self.physical_devices.iter() {
            if dev.suitable && dev.is_complete() {
                let properties = unsafe { self.instance.get_physical_device_properties(dev.phy) };
                match selected_device {
                    Some(_) => {
                        if selected_device_type != vk::PhysicalDeviceType::DISCRETE_GPU
                            && (properties.device_type == vk::PhysicalDeviceType::INTEGRATED_GPU
                                || properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU
                                || properties.device_type == vk::PhysicalDeviceType::VIRTUAL_GPU)
                        {
                            selected_device = Some(index);
                            selected_device_type = properties.device_type;
                        }
                    }
                    None => selected_device = Some(index),
                }
            }
            index += 1;
        }
        match selected_device {
            Some(idx) => self.create_graphic_device(idx),
            None => Err(errors::VulkanError::DeviceSelectionError),
        }
    }

    pub fn create_graphic_device(
        &self,
        index: usize,
    ) -> Result<device::Device, errors::VulkanError> {
        let graphics_device = self.physical_devices.get(index);

        match graphics_device {
            Some(dev) => device::Device::new(&self.instance, &dev.phy),
            None => Err(errors::VulkanError::DeviceSelectionError),
        }
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
            self.surface_fn.destroy_surface(self.surface, None);
            self.instance.destroy_instance(None);
        }
    }
}
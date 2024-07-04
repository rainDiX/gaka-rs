/*
* SPDX-License-Identifier: MIT
*/

use ash::vk;

struct Device {
    phy: vk::PhysicalDevice,
    logical_device: ash::Device,
    graphics_queue: vk::Queue,
}

impl Device {

}

/*
* SPDX-License-Identifier: MIT
*/

use nalgebra_glm as glm;
use std::collections::HashMap;

use super::{camera::FlyingCamera, lights::PointLight};
use crate::rendering::RenderObject;

pub struct Scene {
    camera: FlyingCamera,
    objects: HashMap<String, RenderObject>,
    point_lights: Vec<(PointLight, glm::Vec3)>,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            camera: FlyingCamera::new(
                glm::vec3(0.0, 2.0, 5.0),
                glm::vec3(0.0, 2.0, 4.0),
                glm::vec3(0.0, 1.0, 0.0),
            ),
            objects: HashMap::new(),
            point_lights: Vec::new(),
        }
    }

    pub fn add_point_light(&mut self,light: PointLight, position: glm::Vec3) {
        self.point_lights.push((light, position));
    }

    pub fn point_lights(&self) -> &Vec<(PointLight, glm::Vec3)> {
        &self.point_lights
    }

    pub fn add_object(&mut self, handle: &str, object: RenderObject) {
        self.objects.insert(handle.to_string(), object);
    }

    pub fn get_object(&self, handle: &str) -> Option<&RenderObject> {
        self.objects.get(handle)
    }

    pub fn get_object_mut(&mut self, handle: &str) -> Option<&mut RenderObject> {
        self.objects.get_mut(handle)
    }

    pub fn remove_object(&mut self, handle: &str) {
        self.objects.remove(handle);
    }

    pub fn objects(&self) -> &HashMap<String, RenderObject> {
        &self.objects
    }

    pub fn camera(&self) -> &FlyingCamera {
        &self.camera
    }

    pub fn get_camera_mut(&mut self) -> &mut FlyingCamera {
        &mut self.camera
    }

}

/*
* SPDX-License-Identifier: MIT
*/

use nalgebra_glm as glm;

pub trait Camera {
    fn get_view_matrix(&self) -> glm::Mat4;
}

pub struct FlyingCamera {
    position: glm::Vec3,
    view_point: glm::Vec3,
    up: glm::Vec3,
    fov: f32,
}

impl FlyingCamera {
    pub fn new(position: glm::Vec3, view_point: glm::Vec3, up: glm::Vec3) -> Self {
        Self {
            position,
            view_point,
            up,
            fov: 45.0
        }
    }

    pub fn fov(&self) -> f32 {
        self.fov
    }

    pub fn set_fov(&mut self, fov: f32) {
        self.fov = fov;
    }

    pub fn move_by(&mut self, distance: f32) {
        let offset = self.get_normalized_view_vector() * distance;
        self.position += offset;
        self.view_point += offset;
    }

    pub fn move_up(&mut self, distance: f32) {
        let offset = self.up * distance;
        self.position += offset;
        self.view_point += offset;
    }

    pub fn strafe_by(&mut self, distance: f32) {
        let strafe_vector =
            glm::normalize(&glm::cross(&self.get_normalized_view_vector(), &self.up)) * distance;

        self.position += strafe_vector;
        self.view_point += strafe_vector;
    }

    fn get_normalized_view_vector(&self) -> glm::Vec3 {
        glm::normalize(&(self.view_point - self.position))
    }

    fn rotate_by(&mut self, angle: f32, axis: glm::Vec3) {
        let rotation_matrix = glm::Mat4::new_rotation(angle.to_radians() * axis);
        let rotated_view_vector =
            rotation_matrix * glm::vec3_to_vec4(&self.get_normalized_view_vector());
        self.view_point = self.position + glm::vec4_to_vec3(&rotated_view_vector);
    }

    pub fn rotate_right(&mut self, angle: f32) {
        self.rotate_by(angle, glm::vec3(0.0, 1.0, 0.0));
    }

    pub fn rotate_up(&mut self, angle: f32) {
        self.rotate_by(angle, glm::vec3(0.0, 0.0, 1.0));
    }

}

impl Camera for FlyingCamera {
    fn get_view_matrix(&self) -> glm::Mat4 {
        glm::look_at(&self.position, &self.view_point, &self.up)
    }
}

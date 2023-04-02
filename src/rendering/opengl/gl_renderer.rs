/*
* SPDX-License-Identifier: MIT
*/

use crate::asset_manager::AssetManager;
use crate::gl_check;
use crate::rendering::camera::Camera;
use crate::rendering::scene::Scene;
use crate::rendering::{ShaderProgram, ShaderType};

use glutin::prelude::GlDisplay;
use std::collections::HashMap;
use std::ffi::CString;
use std::rc::Rc;

use super::show_platform_informations;
use nalgebra_glm as glm;

pub struct GlRenderer {
    scene: Scene,
    programs: HashMap<String, Rc<ShaderProgram>>,
    asset_manager: AssetManager,
    aspect_ratio: f32,
}

impl GlRenderer {
    pub fn new<D: GlDisplay>(gl_display: &D, asset_manager: AssetManager) -> Self {
        unsafe {
            gl::load_with(|symbol| {
                let symbol = CString::new(symbol).unwrap();
                gl_display.get_proc_address(symbol.as_c_str()).cast()
            });

            #[cfg(debug_assertions)]
            show_platform_informations();

            gl::Enable(gl::LINE_SMOOTH);
            gl::Hint(gl::LINE_SMOOTH_HINT, gl::NICEST);

            Self {
                scene: Scene::new(),
                programs: HashMap::new(),
                asset_manager,
                aspect_ratio: 1.0,
            }
        }
    }

    // temporary
    pub fn compile_shaders(&mut self) {
        let mut normal = ShaderProgram::new();
        normal
            .compile_file("shaders/mesh.vert", ShaderType::Vertex, &self.asset_manager)
            .expect("Fail to compile File");
        normal
            .compile_file(
                "shaders/normals.frag",
                ShaderType::Fragment,
                &self.asset_manager,
            )
            .expect("Fail to compile File");
        normal.link().expect("Failed to Link Program");

        let mut phong = ShaderProgram::new();
        phong
            .compile_file("shaders/mesh.vert", ShaderType::Vertex, &self.asset_manager)
            .expect("Fail to compile File");
        phong
            .compile_file(
                "shaders/phong.frag",
                ShaderType::Fragment,
                &self.asset_manager,
            )
            .expect("Fail to compile File");
        phong.link().expect("Failed to Link Program");

        self.programs.insert("normal".to_owned(), Rc::new(normal));
        self.programs.insert("phong".to_owned(), Rc::new(phong));
    }

    pub fn render_scene(&self) {
        unsafe {
            gl_check!(gl::ClearColor(0.1, 0.1, 0.1, 1.0));
            gl_check!(gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT));
        }
        let camera = self.scene.camera();
        let projection =
            glm::perspective(camera.fov().to_radians(), self.aspect_ratio, 0.5, 1000.0);
        let model = glm::identity::<f32, 4>();
        // TODO for now the modelMatrix is just the identity
        // model = glm::rotate(&model, angle.to_radians(), glm::vec3(1.0, 0.3, 0.5));
        // model = glm::translate(&model, &glm::vec3(0.0, 0.0, 0.0));
        for (_, object) in self.scene.objects() {
            object.draw(
                &projection,
                &camera.get_view_matrix(),
                &model,
                self.scene.point_lights(),
            );
        }
    }

    pub fn get_scene(&self) -> &Scene {
        &self.scene
    }

    pub fn get_scene_mut(&mut self) -> &mut Scene {
        &mut self.scene
    }

    pub fn get_program(&self, handle: &str) -> Option<&Rc<ShaderProgram>> {
        self.programs.get(handle)
    }

    pub fn set_viewport(&mut self, x: i32, y: i32, width: i32, height: i32) {
        self.aspect_ratio = width as f32 / height as f32;
        unsafe {
            gl_check!(gl::Viewport(x, y, width, height));
        }
    }

    pub fn resize(&mut self, width: i32, height: i32) {
        self.set_viewport(0, 0, width, height);
        self.aspect_ratio = width as f32 / height as f32;
    }
}

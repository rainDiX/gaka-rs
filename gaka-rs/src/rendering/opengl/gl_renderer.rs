/*
* SPDX-License-Identifier: MIT
*/

use crate::rendering::vertex::Vertices;
use crate::{asset_manager::AssetManager, gl_check};

use glutin::prelude::GlDisplay;
use std::ffi::CString;

use super::gl_program::{ShaderProgram, ShaderType};
use super::gl_utils::show_platform_informations;
use super::gl_vertex::GlVertices;

pub struct GlRenderer {
    program: ShaderProgram,
    vertices: GlVertices,
}

impl GlRenderer {
    pub fn new<D: GlDisplay, T>(
        gl_display: &D,
        asset_manager: &AssetManager,
        vertices: &Vertices<T>,
    ) -> Self {
        unsafe {
            gl::load_with(|symbol| {
                let symbol = CString::new(symbol).unwrap();
                gl_display.get_proc_address(symbol.as_c_str()).cast()
            });

            #[cfg(debug_assertions)]
            show_platform_informations();

            let mut program = ShaderProgram::new();

            program
                .compile_file("shaders/curve.vert", ShaderType::Vertex, &asset_manager)
                .expect("Fail to compile File");
            program
                .compile_file("shaders/curve.frag", ShaderType::Fragment, &asset_manager)
                .expect("Fail to compile File");

            program.link().expect("Failed to Link Program");

            let vertices = GlVertices::new(&vertices, &mut program);

            gl::Enable(gl::LINE_SMOOTH);
            Self { program, vertices }
        }
    }

    pub fn draw(&self) {
        unsafe {
            self.program.activate().expect("Fail to use program");

            self.vertices.bind();

            gl_check!(gl::ClearColor(0.1, 0.1, 0.1, 1.0));
            gl_check!(gl::Clear(gl::COLOR_BUFFER_BIT));
            // gl_check!(gl::DrawArrays(gl::TRIANGLES, 0, 3));
            gl_check!(gl::DrawElements(
                gl::LINES,
                self.vertices.index_count(),
                gl::UNSIGNED_INT,
                std::ptr::null()
            ));
        }
    }

    pub fn set_viewport(&self, x: i32, y: i32, width: i32, height: i32) {
        unsafe {
            gl_check!(gl::Viewport(x, y, width, height));
        }
    }

    pub fn resize(&self, width: i32, height: i32) {
        self.set_viewport(0, 0, width, height);
    }
}

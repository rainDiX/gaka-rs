/*
* SPDX-License-Identifier: MIT
*/

extern crate gl;
extern crate nalgebra_glm as glm;

use crate::{asset_manager::AssetManager, gl_check};

use glutin::prelude::GlDisplay;
use std::ffi::CString;

use super::program::{ShaderProgram, ShaderType};
use super::utils::show_platform_informations;

pub struct GlRenderer {
    program: ShaderProgram,
    vao: gl::types::GLuint,
    vbo: gl::types::GLuint,
}

impl GlRenderer {
    pub fn new<D: GlDisplay>(
        gl_display: &D,
        asset_manager: &AssetManager,
        vertex_data: &[f32],
    ) -> Self {
        unsafe {
            gl::load_with(|symbol| {
                let symbol = CString::new(symbol).unwrap();
                gl_display.get_proc_address(symbol.as_c_str()).cast()
            });

            #[cfg(debug_assertions)]
            show_platform_informations();

            let mut program = ShaderProgram::new();

            program.compile_file(
                "shaders/vertexShader.vert",
                ShaderType::Vertex,
                &asset_manager,
            ).expect("Fail to compile File");
            program.compile_file(
                "shaders/fragmentShader.frag",
                ShaderType::Fragment,
                &asset_manager,
            ).expect("Fail to compile File");

            program.link().expect("Failed to Link Program");

            program.set_used();

            let mut vao = std::mem::zeroed();
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);

            let mut vbo = std::mem::zeroed();
            gl::GenBuffers(1, &mut vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (vertex_data.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
                vertex_data.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );

            let pos_attrib = gl::GetAttribLocation(program.id(), b"position\0".as_ptr() as *const _);
            let color_attrib = gl::GetAttribLocation(program.id(), b"color\0".as_ptr() as *const _);

            gl::VertexAttribPointer(
                pos_attrib as gl::types::GLuint,
                3,
                gl::FLOAT,
                gl::FALSE,
                6 * std::mem::size_of::<f32>() as gl::types::GLsizei,
                std::ptr::null(),
            );
            gl::VertexAttribPointer(
                color_attrib as gl::types::GLuint,
                3,
                gl::FLOAT,
                0,
                6 * std::mem::size_of::<f32>() as gl::types::GLsizei,
                (3 * std::mem::size_of::<f32>()) as *const () as *const _,
            );
            gl::EnableVertexAttribArray(pos_attrib as gl::types::GLuint);
            gl::EnableVertexAttribArray(color_attrib as gl::types::GLuint);

            Self { program, vao, vbo }
        }
    }

    pub fn draw(&self) {
        unsafe {
            self.program.set_used().expect("Fail to use program");

            gl::BindVertexArray(self.vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);

            gl_check!(gl::ClearColor(0.1, 0.1, 0.1, 1.0));
            gl_check!(gl::Clear(gl::COLOR_BUFFER_BIT));
            gl_check!(gl::DrawArrays(gl::TRIANGLES, 0, 3));
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

impl Drop for GlRenderer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.vbo);
            gl::DeleteVertexArrays(1, &self.vao);
        }
    }
}

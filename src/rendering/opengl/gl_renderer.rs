/*
* SPDX-License-Identifier: MIT
*/

extern crate gl;
extern crate nalgebra_glm as glm;

use crate::{asset_manager::AssetsManager, gl_check};

use glutin::prelude::GlDisplay;
use std::ffi::{CStr, CString};

use super::utils::gl_comp_status;

pub struct GlRenderer {
    program: gl::types::GLuint,
    vao: gl::types::GLuint,
    vbo: gl::types::GLuint,
}

impl GlRenderer {
    pub fn new<D: GlDisplay>(
        gl_display: &D,
        asset_manager: &AssetsManager,
        vertex_data: &[f32],
    ) -> Self {
        unsafe {
            gl::load_with(|symbol| {
                let symbol = CString::new(symbol).unwrap();
                gl_display.get_proc_address(symbol.as_c_str()).cast()
            });

            #[cfg(debug_assertions)]
            show_platform_informations();

            let vertex_shader_source = asset_manager.read_cstring("shaders/vertexShader.vert").unwrap();
            let fragment_shader_source = asset_manager.read_cstring("shaders/fragmentShader.frag").unwrap();

            let vertex_shader = create_shader(gl::VERTEX_SHADER, vertex_shader_source.as_bytes());
            let fragment_shader = create_shader(gl::FRAGMENT_SHADER, fragment_shader_source.as_bytes());

            let program = gl::CreateProgram();

            gl::AttachShader(program, vertex_shader);
            gl::AttachShader(program, fragment_shader);

            gl::LinkProgram(program);

            gl::UseProgram(program);

            gl::DeleteShader(vertex_shader);
            gl::DeleteShader(fragment_shader);

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

            let pos_attrib = gl::GetAttribLocation(program, b"position\0".as_ptr() as *const _);
            let color_attrib = gl::GetAttribLocation(program, b"color\0".as_ptr() as *const _);

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
            gl::UseProgram(self.program);

            gl::BindVertexArray(self.vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);

            gl_check!(gl::ClearColor(0.1, 0.1, 0.1, 1.0));
            gl_check!(gl::Clear(gl::COLOR_BUFFER_BIT));
            gl_check!(gl::DrawArrays(gl::TRIANGLES, 0, 3));
        }
    }

    pub fn resize(&self, width: i32, height: i32) {
        unsafe {
            gl_check!(gl::Viewport(0, 0, width, height));
        }
    }
}

impl Drop for GlRenderer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.program);
            gl::DeleteBuffers(1, &self.vbo);
            gl::DeleteVertexArrays(1, &self.vao);
        }
    }
}

unsafe fn create_shader(shader: gl::types::GLenum, source: &[u8]) -> gl::types::GLuint {
    let shader = gl::CreateShader(shader);
    gl::ShaderSource(
        shader,
        1,
        [source.as_ptr().cast()].as_ptr(),
        std::ptr::null(),
    );
    gl::CompileShader(shader);

    gl_comp_status(shader);

    shader
}

#[cfg(debug_assertions)]
fn show_platform_informations() {
    if let Some(renderer) = get_gl_string(gl::RENDERER) {
        eprintln!("Running on {}", renderer.to_string_lossy());
    }
    if let Some(version) = get_gl_string(gl::VERSION) {
        eprintln!("OpenGL Version {}", version.to_string_lossy());
    }

    if let Some(shaders_version) = get_gl_string(gl::SHADING_LANGUAGE_VERSION) {
        eprintln!("Shaders version on {}", shaders_version.to_string_lossy());
    }
}

#[cfg(debug_assertions)]
fn get_gl_string(variant: gl::types::GLenum) -> Option<&'static CStr> {
    unsafe {
        let s = gl::GetString(variant);
        (!s.is_null()).then(|| CStr::from_ptr(s.cast()))
    }
}

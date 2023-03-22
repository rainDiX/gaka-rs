/*
* SPDX-License-Identifier: MIT
*/

pub mod gl_renderer;
pub mod gl_program;
pub mod gl_objects;
pub mod gl_texture;

extern crate gl;

use std::{ffi::CStr};

#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! gl_check {
    ($fun: expr) => {{
        $fun
    }};
}

#[cfg(debug_assertions)]
#[macro_export]
macro_rules! gl_check {
    ($fun:expr) => {{
        let (val, err) = ($fun, gl::GetError());
        if err != gl::NO_ERROR {
            let error: String = match err {
                gl::INVALID_ENUM => "GL_INVALID_ENUM".into(),
                gl::INVALID_VALUE => "GL_INVALID_VALUE".into(),
                gl::INVALID_OPERATION => "GL_INVALID_OPERATION".into(),
                gl::STACK_OVERFLOW => "GL_STACK_OVERFLOW".into(),
                gl::STACK_UNDERFLOW => "GL_STACK_UNDERFLOW".into(),
                gl::OUT_OF_MEMORY => "GL_OUT_OF_MEMORY".into(),
                gl::INVALID_FRAMEBUFFER_OPERATION => "GL_INVALID_FRAMEBUFFER_OPERATION".into(),
                gl::CONTEXT_LOST => "GL_CONTEXT_LOST".into(),
                _ => format!("GL_?_{}", err).into(),
            };
            log::error!("OpenGL error {} in {}", error, stringify!($fun));
        }
        val
    }};
}

fn gl_info_log_to_string(info_log: &mut Vec<i8>, len: i32) -> String {
    let log = unsafe {
        info_log.set_len(len as usize);
        std::slice::from_raw_parts(info_log.as_ptr() as *const u8, info_log.len())
    };
    String::from_utf8(log.to_vec()).expect("Found invalid UTF-8")
}

fn get_gl_string(variant: gl::types::GLenum) -> Option<&'static CStr> {
    unsafe {
        let s = gl::GetString(variant);
        (!s.is_null()).then(|| CStr::from_ptr(s.cast()))
    }
}

fn show_platform_informations() {
    if let Some(renderer) = get_gl_string(gl::RENDERER) {
        log::info!("Running on {}", renderer.to_string_lossy());
    }
    if let Some(version) = get_gl_string(gl::VERSION) {
        log::info!("OpenGL Version {}", version.to_string_lossy());
    }

    if let Some(shaders_version) = get_gl_string(gl::SHADING_LANGUAGE_VERSION) {
        log::info!("Shaders version on {}", shaders_version.to_string_lossy());
    }
}

fn components_type_size(type_enum : gl::types::GLenum) -> (i32, gl::types::GLenum, usize) {
    match type_enum {
        gl::FLOAT => (1, gl::FLOAT, std::mem::size_of::<gl::types::GLfloat>()),
        gl::FLOAT_VEC2 => (2, gl::FLOAT, 2* std::mem::size_of::<gl::types::GLfloat>()),
        gl::FLOAT_VEC3 => (3, gl::FLOAT, 3 * std::mem::size_of::<gl::types::GLfloat>()),
        gl::FLOAT_VEC4 => (4, gl::FLOAT, 4 * std::mem::size_of::<gl::types::GLfloat>()),
        gl::INT => (1, gl::INT, std::mem::size_of::<gl::types::GLint>()),
        gl::INT_VEC2 => (2, gl::INT, 2* std::mem::size_of::<gl::types::GLint>()),
        gl::INT_VEC3 => (3, gl::INT, 3* std::mem::size_of::<gl::types::GLint>()),
        gl::INT_VEC4 => (4, gl::INT, 4* std::mem::size_of::<gl::types::GLint>()),
        gl::UNSIGNED_INT => (1, gl::UNSIGNED_INT, std::mem::size_of::<gl::types::GLuint>()),
        gl::UNSIGNED_INT_VEC2 => (2, gl::UNSIGNED_INT, 2* std::mem::size_of::<gl::types::GLuint>()),
        gl::UNSIGNED_INT_VEC3 => (3, gl::UNSIGNED_INT, 3* std::mem::size_of::<gl::types::GLuint>()),
        gl::UNSIGNED_INT_VEC4 => (4, gl::UNSIGNED_INT, 4* std::mem::size_of::<gl::types::GLuint>()),
        gl::DOUBLE => (1, gl::DOUBLE, std::mem::size_of::<gl::types::GLdouble>()),
        gl::BOOL => (1, gl::BOOL, std::mem::size_of::<gl::types::GLboolean>()),
        _ => (0, 0, 0),
    }
}

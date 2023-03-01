/*
* SPDX-License-Identifier: MIT
*/

extern crate gl;

use std::ffi::CStr;

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

pub fn gl_info_log_to_string(info_log: &mut Vec<i8>, len: i32) -> String {
    let log = unsafe {
        info_log.set_len(len as usize);
        std::slice::from_raw_parts(info_log.as_ptr() as *const u8, info_log.len())
    };
    String::from_utf8(log.to_vec()).expect("Found invalid UTF-8")
}

pub fn get_gl_string(variant: gl::types::GLenum) -> Option<&'static CStr> {
    unsafe {
        let s = gl::GetString(variant);
        (!s.is_null()).then(|| CStr::from_ptr(s.cast()))
    }
}

pub fn show_platform_informations() {
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

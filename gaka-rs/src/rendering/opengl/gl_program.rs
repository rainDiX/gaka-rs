/*
* SPDX-License-Identifier: MIT
*/

use std::ffi::CString;

extern crate gl;
use super::gl_utils::gl_info_log_to_string;
use crate::{asset_manager::AssetManager, gl_check};

use std::collections::HashMap;

use gl::types::{GLenum, GLint, GLuint, GLfloat};

#[repr(u32)]
pub enum ShaderType {
    Vertex = gl::VERTEX_SHADER,
    Fragment = gl::FRAGMENT_SHADER,
    Geometry = gl::GEOMETRY_SHADER,
    Compute = gl::COMPUTE_SHADER,
    TessEvaluation = gl::TESS_EVALUATION_SHADER,
    TessControl = gl::TESS_CONTROL_SHADER,
}

#[derive(Debug)]
pub enum ProgramError {
    NotLinked,
    LinkingFailed,
    ShaderCompilationFailed,
    ShaderReadingFailed,
}

pub struct ShaderProgram {
    id: GLuint,
    linked: bool,
    attribute_locations: HashMap<String, GLuint>,
    uniform_locations: HashMap<String, GLint>,
}

impl ShaderProgram {
    pub fn new() -> Self {
        let id: GLuint;
        unsafe {
            id = gl::CreateProgram();
        }
        Self {
            id,
            linked: false,
            attribute_locations: HashMap::new(),
            uniform_locations: HashMap::new(),
        }
    }

    #[inline]
    pub fn id(&self) -> GLuint {
        self.id
    }

    #[inline]
    pub fn is_linked(&self) -> bool {
        self.linked
    }

    #[inline]
    pub fn activate(&self) -> Result<(), ProgramError> {
        if !self.linked {
            Err(ProgramError::NotLinked)
        } else {
            unsafe {
                gl_check!(gl::UseProgram(self.id));
            }
            Ok(())
        }
    }

    pub fn set_bool(&mut self, name: &str, value: bool)
    {     
        unsafe{
            gl::Uniform1i(self.get_uniform_location(name),value as GLint); 
        }    
    }

    pub fn set_int(&mut self, name: &str, value: GLint)
    {     
        unsafe{
            gl::Uniform1i(self.get_uniform_location(name),value); 
        }    
    }

    pub fn set_float(&mut self, name: &str, value: GLfloat)
    {     
        unsafe{
            gl::Uniform1f(self.get_uniform_location(name),value); 
        }    
    }

    fn get_uniform_location(&mut self, name: &str) -> GLint {
        match self.uniform_locations.get(name) {
            Some(location) => *location,
            None => unsafe {
                let cname = CString::new(name).expect("Failed to convert name to CString");
                let location =
                    gl::GetUniformLocation(self.id, cname.to_bytes_with_nul().as_ptr() as *const _);
                self.uniform_locations.insert(name.to_string(), location);
                location
            },
        }
    }

    pub fn get_attribute_location(&mut self, name: &String) -> GLuint {
        match self.attribute_locations.get(name) {
            Some(location) => *location,
            None => unsafe {
                let cname = CString::new(name.clone()).expect("Failed to convert name to CString");
                let location =
                    gl::GetAttribLocation(self.id, cname.to_bytes_with_nul().as_ptr() as *const _);
                self.uniform_locations.insert(name.clone(), location);
                location as GLuint
            },
        }
    }

    pub fn compile_source(
        &mut self,
        source: &CString,
        shader_type: GLenum,
    ) -> Result<(), ProgramError> {
        unsafe {
            let shader = gl::CreateShader(shader_type);
            gl::ShaderSource(
                shader,
                1,
                [source.as_ptr().cast()].as_ptr(),
                std::ptr::null(),
            );
            gl::CompileShader(shader);

            let mut success: GLint = 0;
            gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);

            if success == 0 {
                #[cfg(debug_assertions)]
                gl_shader_log(shader);

                Err(ProgramError::ShaderCompilationFailed)
            } else {
                gl::AttachShader(self.id, shader);
                Ok(())
            }
        }
    }

    pub fn compile_file(
        &mut self,
        rel_path: &str,
        shader_type: ShaderType,
        asset_manager: &AssetManager,
    ) -> Result<(), ProgramError> {
        match asset_manager.read_cstring(rel_path) {
            Ok(source) => self.compile_source(&source, shader_type as GLenum),
            Err(_) => Err(ProgramError::ShaderReadingFailed),
        }
    }

    unsafe fn delete_shaders(&self) {
        let mut shader_count: GLint = 0;
        gl::GetProgramiv(self.id, gl::ATTACHED_SHADERS, &mut shader_count);
        let mut shaders: Vec<GLuint> = Vec::with_capacity(shader_count as usize);

        gl::GetAttachedShaders(
            self.id,
            shader_count,
            std::ptr::null_mut(),
            shaders.as_mut_ptr(),
        );

        for shader in shaders.iter() {
            gl_check!(gl::DeleteShader(*shader));
        }
    }

    pub fn link(&mut self) -> Result<(), ProgramError> {
        if self.linked {
            return Ok(());
        }
        unsafe {
            gl::LinkProgram(self.id);

            let mut success: GLint = 0;
            gl::GetProgramiv(self.id, gl::LINK_STATUS, &mut success);

            if success == 0 {
                #[cfg(debug_assertions)]
                gl_program_log(self.id);

                Err(ProgramError::LinkingFailed)
            } else {
                self.linked = true;
                self.delete_shaders();
                Ok(())
            }
        }
    }
}

impl Drop for ShaderProgram {
    fn drop(&mut self) {
        unsafe {
            gl_check!(gl::DeleteProgram(self.id));
        }
    }
}

#[cfg(debug_assertions)]
unsafe fn gl_program_log(program: GLuint) {
    let mut len: GLint = 0;
    gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
    let mut info_log: Vec<i8> = Vec::with_capacity(len as usize + 1);
    gl::GetProgramInfoLog(program, len, std::ptr::null_mut(), info_log.as_mut_ptr());
    log::error!("{}", gl_info_log_to_string(&mut info_log, len));
}

#[cfg(debug_assertions)]
unsafe fn gl_shader_log(shader: GLuint) {
    let mut len: GLint = 0;
    gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
    let mut info_log: Vec<i8> = Vec::with_capacity(len as usize + 1);
    gl::GetShaderInfoLog(shader, len, std::ptr::null_mut(), info_log.as_mut_ptr());
    log::error!("{}", gl_info_log_to_string(&mut info_log, len));
}
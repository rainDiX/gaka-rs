/*
* SPDX-License-Identifier: MIT
*/

use std::ffi::CString;

extern crate gl;
use super::components_type_size;
use super::gl_info_log_to_string;
use crate::{
    asset_manager::AssetManager,
    gl_check,
    rendering::{SetUniform, VertexAttribute},
};

use nalgebra_glm as glm;

use gl::types::{GLenum, GLfloat, GLint, GLuint};
use glm::{Mat3, Mat4, Vec2, Vec3, Vec4};

#[repr(u32)]
pub enum GlShaderType {
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

pub struct GlShaderProgram {
    id: GLuint,
    linked: bool,
    attributes: Vec<VertexAttribute>,
}

impl GlShaderProgram {
    pub fn new() -> Self {
        let id: GLuint;
        unsafe {
            id = gl::CreateProgram();
        }
        Self {
            id,
            linked: false,
            attributes: Vec::new(),
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

    fn get_uniform_location(&self, name: &str) -> GLint {
        unsafe {
            let cname = CString::new(name.clone()).expect("Failed to convert name to CString");
            gl::GetUniformLocation(self.id, cname.to_bytes_with_nul().as_ptr() as *const _)
        }
    }

    pub fn get_attribute_location(&self, name: &str) -> GLuint {
        unsafe {
            let cname = CString::new(name.clone()).expect("Failed to convert name to CString");
            gl::GetAttribLocation(self.id, cname.to_bytes_with_nul().as_ptr() as *const _) as GLuint
        }
    }

    pub fn get_attributes(&self) -> &[VertexAttribute] {
        &self.attributes
    }

    pub fn compile_source(
        &self,
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
        &self,
        rel_path: &str,
        shader_type: GlShaderType,
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

    unsafe fn update_attributes(&mut self) {
        let mut active_attrs: GLint = 0;

        // Requires OpenGL 4.3+
        gl::GetProgramInterfaceiv(
            self.id,
            gl::PROGRAM_INPUT,
            gl::ACTIVE_RESOURCES,
            &mut active_attrs,
        );

        let properties: Vec<GLenum> = vec![gl::NAME_LENGTH, gl::TYPE];
        let mut values: Vec<GLint> = Vec::with_capacity(properties.len());
        values.resize(properties.len(), 0);
        let mut name_bytes: Vec<u8> = Vec::new();

        let mut offset: usize = 0;

        for attrib in 0..active_attrs as u32 {
            gl::GetProgramResourceiv(
                self.id,
                gl::PROGRAM_INPUT,
                attrib,
                properties.len() as i32,
                properties.as_ptr(),
                values.len() as i32,
                std::ptr::null_mut(),
                values.as_mut_ptr(),
            );

            name_bytes.resize(values[0] as usize, 0);

            gl::GetProgramResourceName(
                self.id,
                gl::PROGRAM_INPUT,
                attrib,
                name_bytes.len() as i32,
                std::ptr::null_mut(),
                name_bytes.as_mut_ptr() as *mut i8,
            );
            let (num, gl_type, size) = components_type_size(values[1] as u32);
            let cname = CString::from_vec_with_nul(name_bytes.clone()).unwrap();
            self.attributes.push(VertexAttribute {
                name: cname
                    .to_str()
                    .expect("Fail to convert CString to String")
                    .to_string(),
                size: num,
                stride: 0,
                offset: offset,
                type_enum: gl_type,
            });
            offset += size;
        }
        for attrib in &mut self.attributes {
            attrib.stride = offset;
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
                self.update_attributes();
                self.delete_shaders();
                Ok(())
            }
        }
    }
}

impl SetUniform<bool> for GlShaderProgram {
    fn set_uniform(&self, name: &str, value: &bool) {
        unsafe {
            gl_check!(gl::Uniform1i(
                self.get_uniform_location(name),
                *value as GLint
            ));
        }
    }
}

impl SetUniform<GLint> for GlShaderProgram {
    fn set_uniform(&self, name: &str, value: &GLint) {
        unsafe {
            gl_check!(gl::Uniform1i(self.get_uniform_location(name), *value));
        }
    }
}

impl SetUniform<GLfloat> for GlShaderProgram {
    fn set_uniform(&self, name: &str, value: &GLfloat) {
        unsafe {
            gl_check!(gl::Uniform1f(self.get_uniform_location(name), *value));
        }
    }
}

impl SetUniform<Vec2> for GlShaderProgram {
    fn set_uniform(&self, name: &str, value: &Vec2) {
        unsafe {
            gl_check!(gl::Uniform2f(
                self.get_uniform_location(name),
                value.x,
                value.y
            ));
        }
    }
}

impl SetUniform<Vec3> for GlShaderProgram {
    fn set_uniform(&self, name: &str, value: &Vec3) {
        unsafe {
            gl_check!(gl::Uniform3f(
                self.get_uniform_location(name),
                value.x,
                value.y,
                value.z
            ));
        }
    }
}

impl SetUniform<Vec4> for GlShaderProgram {
    fn set_uniform(&self, name: &str, value: &Vec4) {
        unsafe {
            gl_check!(gl::Uniform4f(
                self.get_uniform_location(name),
                value.x,
                value.y,
                value.z,
                value.w
            ));
        }
    }
}

impl SetUniform<Mat3> for GlShaderProgram {
    fn set_uniform(&self, name: &str, value: &Mat3) {
        unsafe {
            gl_check!(gl::UniformMatrix3fv(
                self.get_uniform_location(name),
                1,
                gl::FALSE,
                value.as_ptr()
            ));
        }
    }
}

impl SetUniform<Mat4> for GlShaderProgram {
    fn set_uniform(&self, name: &str, value: &Mat4) {
        unsafe {
            gl_check!(gl::UniformMatrix4fv(
                self.get_uniform_location(name),
                1,
                gl::FALSE,
                value.as_ptr()
            ));
        }
    }
}

impl Drop for GlShaderProgram {
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

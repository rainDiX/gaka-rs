/*
* SPDX-License-Identifier: MIT
*/

mod opengl;
pub mod lights;
pub mod camera;
pub mod scene;
pub mod material;

pub type Texture = opengl::gl_texture::GlTexture;
pub type Renderer = opengl::gl_renderer::GlRenderer;
pub type ShaderProgram = opengl::gl_program::GlShaderProgram;
pub type ShaderType = opengl::gl_program::GlShaderType;
pub type RenderObject = opengl::gl_objects::GlOject;
pub type DrawingMode = opengl::gl_objects::GlDrawingMode;

pub trait SetUniform<T> {
    fn set_uniform(&self, name: &str, value: T);
}

pub struct VertexAttribute {
    pub name: String,
    pub size: i32,
    pub type_enum: u32,
    pub stride: usize,
    pub offset: usize,
}

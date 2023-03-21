/*
* SPDX-License-Identifier: MIT
*/

mod opengl;
// pub mod vertex;
pub mod mesh;

pub type Texture = opengl::gl_texture::GlTexture;
pub type Renderer = opengl::gl_renderer::GlRenderer;
pub type ShaderProgram = opengl::gl_program::GlShaderProgram;
pub type ShaderType = opengl::gl_program::GlShaderType;
pub type RenderObject = opengl::gl_objects::GlOject;

pub struct VertexAttribute {
    pub name: String,
    pub size: i32,
    pub stride: usize,
    pub offset: usize,
}

pub trait Drawable {
    fn draw(&self);
}
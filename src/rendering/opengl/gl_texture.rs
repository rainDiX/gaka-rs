/*
* SPDX-License-Identifier: MIT
*/

extern crate gl;

use gl::types::{GLuint};

pub enum GlTextureType {
    Diffuse,
    Specular,
}

pub struct GlTexture {
    id: GLuint,
    tx_type: GlTextureType,
    path: String,
}

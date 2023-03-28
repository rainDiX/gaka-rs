/*
* SPDX-License-Identifier: MIT
*/

use std::rc::Rc;

use gl::types::{GLenum, GLint, GLsizei, GLsizeiptr, GLuint};

use crate::{
    geometry::mesh::Mesh,
    gl_check,
    rendering::{material::Material, SetUniform, Texture, VertexAttribute, lights::{PointLight, self}},
};

use super::gl_program::GlShaderProgram;

use nalgebra_glm as glm;

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum GlDrawingMode {
    Points = gl::POINTS,
    Lines = gl::LINES,
    LineLoop = gl::LINE_LOOP,
    LineStrip = gl::LINE_STRIP,
    Triangles = gl::TRIANGLES,
    TriangleStrip = gl::TRIANGLE_STRIP,
    TriangleFan = gl::TRIANGLE_FAN,
    LinesAdjacency = gl::LINES_ADJACENCY,
    LineStripAdjacency = gl::LINE_STRIP_ADJACENCY,
    TrianglesAdjacency = gl::TRIANGLES_ADJACENCY,
    TrianglesStripAdjacency = gl::TRIANGLE_STRIP_ADJACENCY,
}

pub struct GlOject {
    vao: GLuint,
    vbo: GLuint,
    ebo: GLuint,
    index_count: GLint,
    vertex_buffer_size: usize,
    drawing_mode: GlDrawingMode,
    material: Rc<Material>,
    textures: Vec<Texture>,
}

impl GlOject {
    pub fn new(
        mesh: &Mesh,
        program: &GlShaderProgram,
        textures: Vec<Texture>,
        material: Rc<Material>,
    ) -> Self {
        let mut vao: GLuint = 0;
        let mut vbo: GLuint = 0;
        let mut ebo: GLuint = 0;
        setup_vertex_objects(&mut vao, &mut vbo, &mesh.vertices);
        setup_attrib_pointer(program.get_attributes(), &program);
        let index_count = mesh.indices.len() as GLint;
        setup_element_objects(&mut ebo, &mesh.indices);

        Self {
            vao,
            vbo,
            ebo,
            index_count,
            vertex_buffer_size: mesh.vertices.len(),
            drawing_mode: GlDrawingMode::Triangles,
            material,
            textures,
        }
    }

    pub fn bind(&self) {
        unsafe {
            if self.vao > 0 {
                gl_check!(gl::BindVertexArray(self.vao));
            }
        }
    }

    pub fn draw(
        &self,
        projection_matrix: &glm::Mat4,
        view_matrix: &glm::Mat4,
        model_matrix: &glm::Mat4,
        lights: &Vec<(PointLight, glm::Vec3)>,
        program: &GlShaderProgram,
    ) {
        unsafe {
            self.bind();
            program.activate().expect("Fail to use program");

            program.set_uniform("projection", projection_matrix);
            program.set_uniform("view", view_matrix);
            program.set_uniform("model", model_matrix);

            program.set_uniform("material.ambient", &self.material.ambient);
            program.set_uniform("material.diffuse", &self.material.diffuse);
            program.set_uniform("material.specular", &self.material.specular);
            program.set_uniform("material.shininess", self.material.shininess);

            // gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);
            program.set_uniform("nb_point_lights", lights.len() as GLint);
            for i in 0..lights.len() {
                program.set_uniform(&format!("pointLights[{}].color", i), &lights[i].0.color);
                program.set_uniform(&format!("pointLights[{}].intensity", i), lights[i].0.intensity);
                program.set_uniform(&format!("pointLights[{}].range", i), lights[i].0.range);
                program.set_uniform(&format!("pointLights[{}].decay", i), lights[i].0.decay);
                program.set_uniform(&format!("pointLights[{}].position", i), &lights[i].1);
            }

            gl_check!(gl::DrawElements(
                self.drawing_mode as u32,
                self.index_count,
                gl::UNSIGNED_INT,
                std::ptr::null()
            ));
        }
    }

    pub fn update<T>(&mut self, mesh: &Mesh) {
        self.index_count = mesh.indices.len() as GLint;
        unsafe {
            self.vertex_buffer_size = update_buffer(
                self.vbo,
                &mesh.vertices,
                self.vertex_buffer_size,
                gl::ARRAY_BUFFER,
            );
            self.vertex_buffer_size = update_buffer(
                self.ebo,
                &mesh.indices,
                self.vertex_buffer_size,
                gl::ELEMENT_ARRAY_BUFFER,
            );
        };
    }

    pub fn set_drawing_mode(&mut self, mode: GlDrawingMode) {
        self.drawing_mode = mode;
    }

    pub fn drawing_mode(&self) -> GlDrawingMode {
        self.drawing_mode
    }
}

impl Drop for GlOject {
    fn drop(&mut self) {
        unsafe {
            if self.vbo > 0 {
                gl_check!(gl::DeleteBuffers(1, &self.vbo));
            }
            if self.ebo > 0 {
                gl_check!(gl::DeleteBuffers(1, &self.ebo));
            }
            if self.vao > 0 {
                gl_check!(gl::DeleteVertexArrays(1, &self.vao));
            }
        }
    }
}

#[inline]
fn setup_vertex_objects<T>(vao: &mut u32, vbo: &mut u32, v: &[T]) {
    unsafe {
        gl_check!(gl::GenVertexArrays(1, vao));
        gl_check!(gl::BindVertexArray(*vao));
        gl_check!(gl::GenBuffers(1, vbo));
        update_buffer(*vbo, v, 0, gl::ARRAY_BUFFER);
    };
}

#[inline]
fn setup_element_objects(ebo: &mut u32, indices: &[GLuint]) {
    unsafe {
        gl_check!(gl::GenBuffers(1, ebo));
        update_buffer(*ebo, &indices, 0, gl::ELEMENT_ARRAY_BUFFER);
    }
}

#[inline(always)]
unsafe fn update_buffer<T>(
    handle: u32,
    buffer: &[T],
    old_buf_size: usize,
    target: GLenum,
) -> usize {
    gl_check!(gl::BindBuffer(target, handle));
    if old_buf_size != buffer.len() {
        gl_check!(gl::BufferData(
            target,
            (buffer.len() * std::mem::size_of::<T>()) as GLsizeiptr,
            buffer.as_ptr() as *const _,
            gl::STATIC_DRAW,
        ));
    } else {
        gl_check!(gl::BufferSubData(
            target,
            0,
            (buffer.len() * std::mem::size_of::<T>()) as GLsizeiptr,
            buffer.as_ptr() as *const _,
        ));
    }
    buffer.len()
}

#[inline]
fn setup_attrib_pointer(attributes: &[VertexAttribute], program: &GlShaderProgram) {
    unsafe {
        for attrib in attributes {
            let location = program.get_attribute_location(&attrib.name);
            gl_check!(gl::VertexAttribPointer(
                location,
                attrib.size,
                gl::FLOAT,
                gl::FALSE,
                attrib.stride as GLsizei,
                attrib.offset as *const _
            ));
            gl_check!(gl::EnableVertexAttribArray(location));
        }
    }
}

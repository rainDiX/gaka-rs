/*
* SPDX-License-Identifier: MIT
*/
use std::rc::Rc;

use gl::types::{GLenum, GLint, GLsizei, GLsizeiptr, GLuint};

use crate::{gl_check, rendering::VertexAttribute};

use super::gl_program::GlShaderProgram;

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum DrawingMode {
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
    drawing_mode: DrawingMode,
    program: Rc<GlShaderProgram>,
}

impl GlOject {
    pub fn new<T>(
        vertices: &Vec<T>,
        indices: &Vec<GLuint>,
        attributes: &Vec<VertexAttribute>,
        program: Rc<GlShaderProgram>,
    ) -> Self {
        let mut vao: GLuint = 0;
        let mut vbo: GLuint = 0;
        let mut ebo: GLuint = 0;
        setup_vertex_objects(&mut vao, &mut vbo, vertices);
        setup_attrib_pointer(attributes, &program);
        let index_count = indices.len() as GLint;

        if index_count > 0 {
            setup_element_objects(&mut ebo, indices);
        }

        Self {
            vao,
            vbo,
            ebo,
            index_count,
            drawing_mode: DrawingMode::Triangles,
            program,
        }
    }

    pub fn bind(&self) {
        unsafe {
            if self.vao > 0 {
                gl_check!(gl::BindVertexArray(self.vao));
            }
        }
    }

    pub fn draw(&self) {
        unsafe {
            self.bind();
            self.program.activate().expect("Fail to use program");
            if self.index_count > 0 {
                gl_check!(gl::DrawElements(
                    self.drawing_mode as u32,
                    self.index_count,
                    gl::UNSIGNED_INT,
                    std::ptr::null()
                ));
            } else {
                gl_check!(gl::DrawArrays(self.drawing_mode as u32, 0, 3));
            }
        }
    }

    pub fn update<T>(&mut self, vertices: &Vec<T>, indices: &Vec<GLuint>) {
        self.index_count = indices.len() as GLint;
        unsafe {
            update_buffer(self.vbo, &vertices, gl::ARRAY_BUFFER);
            if self.index_count > 0 {
                update_buffer(self.ebo, &indices, gl::ELEMENT_ARRAY_BUFFER);
            }
        };
    }

    pub fn set_drawing_mode(&mut self, mode: DrawingMode) {
        self.drawing_mode = mode;
    }

    pub fn drawing_mode(&self) -> DrawingMode {
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
fn setup_vertex_objects<T>(vao: &mut u32, vbo: &mut u32, v: &Vec<T>) {
    unsafe {
        gl_check!(gl::GenVertexArrays(1, vao));
        gl_check!(gl::BindVertexArray(*vao));
        gl_check!(gl::GenBuffers(1, vbo));
        update_buffer(*vbo, v, gl::ARRAY_BUFFER);
    };
}

#[inline]
fn setup_element_objects(ebo: &mut u32, indices: &Vec<GLuint>) {
    unsafe {
        gl_check!(gl::GenBuffers(1, ebo));
        update_buffer(*ebo, &indices, gl::ELEMENT_ARRAY_BUFFER);
    }
}

#[inline(always)]
unsafe fn update_buffer<T>(handle: u32, buffer: &Vec<T>, target: GLenum) {
    gl_check!(gl::BindBuffer(target, handle));
    gl_check!(gl::BufferData(
        target,
        (buffer.len() * std::mem::size_of::<T>()) as GLsizeiptr,
        buffer.as_ptr() as *const _,
        gl::STATIC_DRAW,
    ));
}

#[inline]
fn setup_attrib_pointer(attributes: &Vec<VertexAttribute>, program: &GlShaderProgram) {
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

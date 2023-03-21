/*
* SPDX-License-Identifier: MIT
*/

use std::rc::Rc;

use nalgebra_glm as glm;

use crate::geometry::{curves::Curve, surfaces::Surface, mesh::{Mesh, Vertex, self}};

use super::{Drawable, RenderObject, ShaderProgram, Texture, VertexAttribute};
use glm::{Vec2, Vec3};

pub struct RenderMesh {
    textures: Vec<Texture>,
    render_object: RenderObject,
}

impl RenderMesh {
    pub fn new(
        mesh: &Mesh,
        textures: Vec<Texture>,
        program: Rc<ShaderProgram>,
    ) -> Self {
        let attributes = Vec::new();
        let render_object = RenderObject::new(&mesh.vertices, &mesh.indices, &attributes, program);
        RenderMesh {
            textures,
            render_object,
        }
    }

    // pub fn update_vertices(&mut self, vertices: &[Mesh], indices: &[u32]) {
    //     self.vertices = vertices.to_vec();
    //     self.indices = indices.to_vec();
    //     self.render_object.update(&self.vertices, &self.indices);
    // }

    // pub fn update_vertices(&mut self, points: &[Vec3], indices: &[u32]) {
    //     self.indices = indices.to_vec();
    //     self.vertices = Vec::with_capacity(points.len());
    //     for i in 0..points.len() {
    //         let vertex = Mesh {
    //             position: points[i].clone(),
    //             normal: Vec3::new(1.0, 1.0, 1.0),
    //             tex_coords: Vec2::new(1.0, 1.0),
    //         };
    //         self.vertices.push(vertex);
    //     }
    //     self.render_object.update(&self.vertices, &self.indices)
    // }

    pub fn from_curve(curve: &impl Curve, program: Rc<ShaderProgram>) -> Self {
        let indices = curve.indices().to_vec();
        let curve = curve.curve();
        let mut vertices = Vec::with_capacity(curve.len());
        for i in 0..curve.len() {
            let vertex = Vertex {
                position: curve[i].clone(),
                normal: Vec3::new(1.0, 1.0, 1.0),
                tex_coords: Vec2::new(1.0, 1.0),
            };
            vertices.push(vertex);
        }
        let textures = Vec::new();
        let mut attributes = Vec::with_capacity(3);

        attributes.push(VertexAttribute {
            name: "position".to_string(),
            size:  3,
            stride: std::mem::size_of::<Vertex>(),
            offset: 0,
        });
        attributes.push(VertexAttribute {
            name: "normal".to_string(),
            size: 3,
            stride: std::mem::size_of::<Vertex>(),
            offset: std::mem::size_of::<Vec3>(),
        });
        attributes.push(VertexAttribute {
            name: "tex_coords".to_string(),
            size: 2,
            stride: std::mem::size_of::<Vertex>(),
            offset: 2 * std::mem::size_of::<Vec3>(),
        });
        let mut render_object = RenderObject::new(&vertices, &indices, &attributes, program);
        render_object.set_drawing_mode(super::opengl::gl_objects::DrawingMode::Lines);
        Self {
            textures,
            render_object,
        }
    }


    pub fn from_surface(surface: &impl Surface, program: Rc<ShaderProgram>) -> Self {
        let textures = Vec::new();
        let mesh = surface.mesh();
        let mut attributes = Vec::with_capacity(3);

        attributes.push(VertexAttribute {
            name: "position".to_string(),
            size:  3,
            stride: std::mem::size_of::<Vertex>(),
            offset: 0,
        });
        attributes.push(VertexAttribute {
            name: "normal".to_string(),
            size: 3,
            stride: std::mem::size_of::<Vertex>(),
            offset: std::mem::size_of::<Vec3>(),
        });
        attributes.push(VertexAttribute {
            name: "tex_coords".to_string(),
            size: 2,
            stride: std::mem::size_of::<Vertex>(),
            offset: 2 * std::mem::size_of::<Vec3>(),
        });
        let mut render_object = RenderObject::new(&mesh.vertices, &mesh.indices, &attributes, program);
        render_object.set_drawing_mode(super::opengl::gl_objects::DrawingMode::Triangles);
        Self {
            textures,
            render_object,
        }
    }
}

impl Drawable for RenderMesh {
    fn draw(&self) {
        // todo: setup texture and uniforms
        self.render_object.draw();
    }
}

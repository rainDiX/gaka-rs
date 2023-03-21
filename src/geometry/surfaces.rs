use nalgebra_glm as glm;

use super::{de_casteljau, Point, Point2D};

use super::curves::{Bezier, Curve};
use super::mesh::{Mesh, Vertex};

pub trait Surface {
    // fn register_2d_point(&mut self, point: Vec2);
    // fn register_3d_point(&mut self, point: Point);
    // fn register_2d_points(&mut self, points: Vec<Vec2>);
    // fn register_3d_points(&mut self, points: Vec<Point>);
    fn mesh(&self) -> &Mesh;
}

pub struct SimpleSurface {
    points: Vec<Point>,
    width: usize,
    height: usize,
}

// impl Surface for SimpleSurface {
//     fn indices(&self) -> &[u32] {}
//     fn normals(&self) -> &[Point] {}
//     fn surface(&self) -> &[Point] {}
// }

pub struct BezierSurface<const M: usize, const N: usize> {
    ctrl_grid: [[Point; N]; M],
    mesh: Mesh,
    mesh_edges: usize,
}

impl<const M: usize, const N: usize> Surface for BezierSurface<M, N> {
    fn mesh(&self) -> &Mesh {
        &self.mesh
    }
}

impl<const M: usize, const N: usize> BezierSurface<M, N> {
    pub fn new(ctrl_grid: [[Point; N]; M], edges: usize) -> Self {
        let mut surface = Self {
            ctrl_grid,
            mesh: Mesh::new(),
            mesh_edges: edges,
        };
        surface.evaluate();
        surface
    }

    pub fn set_epsilon(&mut self, edges: usize) {
        self.mesh_edges = edges;
    }

    fn evaluate(&mut self) {
        // let mut q_points = Vec::with_capacity(M * self.mesh_edges );
        let mut q_points = vec![Point::new(0.0, 0.0, 0.0); M * self.mesh_edges];

        for i in 0..M {
            for v in 0..self.mesh_edges {
                let dv = v as f32 / (self.mesh_edges as f32 - 1.0);
                q_points[v * M + i] = de_casteljau(dv, &self.ctrl_grid[i]);
            }
        }

        for i in 0..(q_points.len() / M) {
            for u in 0..self.mesh_edges {
                let u = u as f32 / (self.mesh_edges as f32 - 1.0);

                let vertex = Vertex {
                    position: de_casteljau(u, &q_points[i * M..(i + 1) * M]),
                    normal: Point::new(0.0, 0.0, 0.0),
                    tex_coords: Point2D::new(1.0, 1.0),
                };
                self.mesh.vertices.push(vertex);
            }
        }
        self.calculate_indices();
        self.calculate_normals();
    }

    fn calculate_indices(&mut self) {
        self.mesh.indices = Vec::with_capacity(self.mesh_edges.pow(2));
        for i in 0..(self.mesh_edges as u32 - 1) {
            for j in 0..(self.mesh_edges as u32 - 1) {
                //first triangle
                self.mesh.indices.push(i * self.mesh_edges as u32 + j);
                self.mesh.indices.push(i * self.mesh_edges as u32 + j + 1);
                self.mesh
                    .indices
                    .push(i * self.mesh_edges as u32 + j + self.mesh_edges as u32);
                //second triangle
                self.mesh.indices.push(i * self.mesh_edges as u32 + j + 1);
                self.mesh
                    .indices
                    .push(i * self.mesh_edges as u32 + j + self.mesh_edges as u32);
                self.mesh
                    .indices
                    .push(i * self.mesh_edges as u32 + j + self.mesh_edges as u32 + 1);
            }
        }
    }

    fn calculate_normals(&mut self) {
        for i in 0..self.mesh.vertices.len() - 3 {
            let p0 = &self.mesh.vertices[self.mesh().indices[i] as usize].position;
            let p1 = &self.mesh.vertices[self.mesh().indices[i + 1] as usize].position;
            let p2 = &self.mesh.vertices[self.mesh().indices[i + 2] as usize].position;

            let v1 = p1 - p0;
            let v2 = p2 - p0;

            let normal = glm::normalize(&glm::cross(&v1, &v2));

            self.mesh.vertices[self.mesh.indices[i] as usize].normal += normal;
            self.mesh.vertices[self.mesh.indices[i + 1] as usize].normal += normal;
            self.mesh.vertices[self.mesh.indices[i + 2] as usize].normal += normal;
        }
    }
}

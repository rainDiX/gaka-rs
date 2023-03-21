/*
* SPDX-License-Identifier: MIT
*/

use super::Point;
use super::Point2D;

pub struct Vertex {
    pub position: Point,
    pub normal: Point,
    pub tex_coords: Point2D,
}

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl Mesh {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }
}

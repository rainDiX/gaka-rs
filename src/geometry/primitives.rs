/*
* SPDX-License-Identifier: MIT
*/

use super::Vec2;
use super::Vec3;

#[repr(C)]
pub struct Line {
    pub start: Vec3,
    pub end: Vec3,
}

#[repr(C)]
pub struct SimpleVertex {
    pub position: Vec3,
    pub normal: Vec3,
}

#[repr(C)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,
}

pub struct PolyLine {
    pub points: Vec<Vec3>,
    pub line_strip: bool
}

impl PolyLine {
    pub fn is_closed(&self) -> bool {
        if let (Some(start), Some(end)) = (self.points.first(), self.points.last()) {
            return start == end;
        }
        false
    }
}
pub struct SimpleMesh {
    pub vertices: Vec<Vertex>,
}

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

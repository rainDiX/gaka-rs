/*
* SPDX-License-Identifier: MIT
*/
use nalgebra_glm as glm;
use glm::{Vec3};

use crate::geometry::curves::Curve;

#[derive(Debug)]
pub enum VertexBuffer<T> {
    Array(Vec<T>),
    Indexed(Vec<T>, Vec<u32>),
}

#[derive(Debug)]
pub struct VertexDesc {
    pub attribute: String,
    pub size: i32,
    pub stride: usize,
    pub offset: usize,
}

pub struct Vertices<T> {
    pub buffer: VertexBuffer<T>,
    pub desc: Vec<VertexDesc>,
}

impl<T> Vertices<T> {
    pub fn new<U>(buffer: VertexBuffer<T>, attributes: Vec<String>) -> Self {
        let mut desc = Vec::new();
        let mut offset = 0;
        let stride = attributes.len();

        for attribute in attributes {
            desc.push(VertexDesc {
                attribute,
                size: (std::mem::size_of::<T>() / std::mem::size_of::<U>()) as i32,
                stride: stride * std::mem::size_of::<T>(),
                offset,
            });
            offset += std::mem::size_of::<T>();
        }
        Self { buffer, desc }
    }
}

impl Vertices<Vec3> {
    pub fn from_curve<C: Curve>(curve: &mut C) -> Self {
        let vertex_data = curve.curve();
        let indices = {
            let mut indices: Vec<u32> = Vec::with_capacity(vertex_data.len() * 2);
            for i in 0..(vertex_data.len() as u32) {
                if i as usize != vertex_data.len() - 1 {
                    indices.push(i);
                    indices.push(i + 1);
                }
            }
            indices
        };
        let desc = vec![VertexDesc {
            attribute: "position".to_owned(),
            size: 3,
            stride: std::mem::size_of::<Vec3>(),
            offset: 0,
        }];
        Self {
            buffer: VertexBuffer::Indexed(vertex_data, indices),
            desc,
        }
    }
}

/*
* SPDX-License-Identifier: MIT
*/

use super::{algorithms::de_casteljau, primitives::PolyLine, Vec3};

#[derive(Clone)]
pub struct Bezier<const N: usize> {
    pub ctrl_points: [Vec3; N],
}

impl<const N: usize> Bezier<N> {
    pub fn new(begin: Vec3, end: Vec3) -> Self {
        let mut ctrl_points = [Vec3::new(0.0, 0.0, 0.0); N];
        for i in 0..N {
            let d: f32 = i as f32 / (N - 1) as f32;
            ctrl_points[i] = begin * (1.0 - d) + end * d;
        }
        Self { ctrl_points }
    }

    pub fn evaluate(&self, resolution: usize) -> PolyLine {
        let mut curve_points = Vec::with_capacity(resolution);
        let mut u: f32 = 0.0;
        let epsilon = 1.0 / resolution as f32;
        while u <= 1.0 {
            curve_points.push(de_casteljau(u, &self.ctrl_points));
            u += epsilon;
        }
        PolyLine {
            points: curve_points,
            line_strip: true,
        }
    }
}

impl<const N: usize> From<[Vec3; N]> for Bezier<N> {
    fn from(points: [Vec3; N]) -> Bezier<N> {
        Self {
            ctrl_points: points,
        }
    }
}

// PieceWiseBezier
pub struct PiecewiseBezier<const N: usize> {
    pub ctrl_points: Vec<Vec3>,
}

impl<const N: usize> PiecewiseBezier<N> {
    pub fn new(begin: Vec3, end: Vec3) -> Self {
        let mut ctrl_points = Vec::with_capacity(N);
        for i in 0..N {
            let d: f32 = i as f32 / (N - 1) as f32;
            ctrl_points[i] = begin * (1.0 - d) + end * d;
        }
        Self {
            ctrl_points: ctrl_points,
        }
    }

    pub fn evaluate(&self, resolution: usize) -> PolyLine {
        let mut curve_points = Vec::with_capacity(resolution);
        let epsilon = 1.0 / (resolution as f32 / self.ctrl_points.len() as f32);
        for i in (0..self.ctrl_points.len()).step_by(N) {
            let mut u: f32 = 0.0;
            while u <= 1.0 {
                curve_points.push(de_casteljau(u, &self.ctrl_points[i..i + N]));
                u += epsilon;
            }
        }
        PolyLine {
            points: curve_points,
            line_strip: true,
        }
    }
}

/*
* SPDX-License-Identifier: MIT
*/

use glam::{Vec2, Vec3, Vec4};

pub trait Curve {
    fn new() -> Self;
    fn register_point2d(&mut self, point: Vec2);
    fn register_point3d(&mut self, point: Vec3);
    fn curve(&mut self) -> Vec<Vec4>;
}

pub struct SimpleCurve {
    points: Vec<Vec4>,
}

impl Curve for SimpleCurve {
    fn new() -> Self {
        Self { points: Vec::new() }
    }

    fn register_point2d(&mut self, point: Vec2) {
        self.points.push(Vec4::from((point, 0.0, 1.0)));
    }

    fn register_point3d(&mut self, point: Vec3) {
        self.points.push(Vec4::from((point, 1.0)));
    }

    fn curve(&mut self) -> Vec<Vec4> {
        self.points.clone()
    }
}

pub struct Bezier {
    //use Vec4 to take advantage of SIMD
    ctrl_points: Vec<Vec4>,
    curve_points: Vec<Vec4>,
    eps: f32,
    constructed: bool,
}

#[inline]
fn de_casteljau(t: f32, points: &Vec<Vec4>) -> Vec4 {
    let mut beta = points.clone();
    let n = points.len();

    for i in 1..n {
        for j in 0..(n - i) {
            beta[j] = beta[j] * (1.0 - t) + beta[j + 1] * t
        }
    }
    beta[0]
}

impl Curve for Bezier {
    fn new() -> Self {
        Self {
            ctrl_points: Vec::new(),
            curve_points: Vec::new(),
            constructed: false,
            eps: 0.01,
        }
    }

    fn register_point2d(&mut self, point: Vec2) {
        self.ctrl_points.push(Vec4::from((point, 0.0, 1.0)));
        self.constructed = false;
    }

    fn register_point3d(&mut self, point: Vec3) {
        self.ctrl_points.push(Vec4::from((point, 1.0)));
        self.constructed = false;
    }

    // Calculating the curve points using De Casteljau algorithm
    fn curve(&mut self) -> Vec<Vec4> {
        if !self.constructed {
            self.curve_points.clear();
            let mut t: f32 = 0.0;
            while t <= 1.0 {
                t += self.eps;
                self.curve_points.push(de_casteljau(t, &self.ctrl_points));
            }
        }
        self.curve_points.clone()
    }
}

impl Bezier {
    #[inline]
    pub fn ctrl_points(&self) -> Vec<Vec4> {
        self.ctrl_points.clone()
    }

    pub fn ctrl_curve(&self) -> SimpleCurve {
        SimpleCurve { points: self.ctrl_points.clone() }
    }

    pub fn set_epsilon(&mut self, eps: f32) {
        self.eps = eps;
        self.constructed = false;
    }

    pub fn epsilon(&self) -> f32 {
        self.eps
    }
}

impl From<Vec<Vec4>> for Bezier {
    fn from(vertices: Vec<Vec4>) -> Self {
        Bezier {
            ctrl_points: vertices,
            curve_points: Vec::new(),
            constructed: false,
            eps: 0.01,
        }
    }
}

impl From<Vec<Vec3>> for Bezier {
    fn from(curve: Vec<Vec3>) -> Self {
        let mut bezier = Bezier::new();
        for vertex in curve {
            bezier.register_point3d(vertex);
        }
        bezier
    }
}

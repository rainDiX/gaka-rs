/*
* SPDX-License-Identifier: MIT
*/
use glm::{Vec2, Vec3};
use nalgebra_glm as glm;

pub trait Curve {
    fn new() -> Self;
    fn register_point2d(&mut self, point: Vec2);
    fn register_point3d(&mut self, point: Vec3);
    fn curve(&mut self) -> Vec<Vec3>;
}

pub struct SimpleCurve {
    points: Vec<Vec3>,
}

impl Curve for SimpleCurve {
    fn new() -> Self {
        Self { points: Vec::new() }
    }

    fn register_point2d(&mut self, point: Vec2) {
        self.points.push(Vec3::new(point.x, point.y, 0.0));
    }

    fn register_point3d(&mut self, point: Vec3) {
        self.points.push(point);
    }

    fn curve(&mut self) -> Vec<Vec3> {
        self.points.clone()
    }
}

impl From<Vec<Vec3>> for SimpleCurve {
    fn from(vertices: Vec<Vec3>) -> Self {
        SimpleCurve { points: vertices }
    }
}

pub struct Bezier {
    ctrl_points: Vec<Vec3>,
    curve_points: Vec<Vec3>,
    eps: f32,
    constructed: bool,
}

#[inline]
fn de_casteljau(t: f32, points: &[Vec3]) -> Vec3 {
    let mut beta = points.to_vec();
    let n = points.len();

    for i in 1..n {
        for j in 0..(n - i) {
            beta[j] = beta[j] * (1.0 - t) + beta[j + 1] * t;
        }
    }
    beta[0]
}

impl Curve for Bezier {
    fn new() -> Self {
        Self {
            ctrl_points: Vec::new(),
            curve_points: Vec::new(),
            constructed: true,
            eps: 0.001,
        }
    }

    fn register_point2d(&mut self, point: Vec2) {
        self.ctrl_points.push(Vec3::new(point.x, point.y, 0.0));
        self.constructed = false;
    }

    fn register_point3d(&mut self, point: Vec3) {
        self.ctrl_points.push(point);
        self.constructed = false;
    }

    // Calculating the curve points using De Casteljau algorithm
    fn curve(&mut self) -> Vec<Vec3> {
        if !self.constructed {
            self.curve_points.clear();
            let mut t: f32 = 0.0;
            let mut i = 0;
            while i < self.ctrl_points.len() {
                let j = std::cmp::min(i + 4, self.ctrl_points.len());
                while t <= 1.0 {
                    t += self.eps;
                    self.curve_points
                        .push(de_casteljau(t, &self.ctrl_points[i..j]));
                }
                t = 0.0;
                i = i + 3;
            }
        }
        self.curve_points.clone()
    }
}

impl Bezier {
    #[inline]
    pub fn ctrl_points(&self) -> Vec<Vec3> {
        self.ctrl_points.clone()
    }

    pub fn ctrl_curve(&self) -> SimpleCurve {
        SimpleCurve {
            points: self.ctrl_points.clone(),
        }
    }

    pub fn set_epsilon(&mut self, eps: f32) {
        self.eps = eps;
        self.constructed = false;
    }

    pub fn epsilon(&self) -> f32 {
        self.eps
    }
}

impl From<Vec<Vec3>> for Bezier {
    fn from(vertices: Vec<Vec3>) -> Self {
        Bezier {
            ctrl_points: vertices,
            curve_points: Vec::new(),
            constructed: false,
            eps: 0.01,
        }
    }
}

/*
* SPDX-License-Identifier: MIT
*/

use super::{de_casteljau, Point, Point2D};

pub trait Curve {
    fn indices(&self) -> &[u32];
    fn curve(&self) -> &[Point];
}

#[derive(Clone)]
pub struct SimpleCurve {
    points: Vec<Point>,
    indices: Vec<u32>,
}

impl Curve for SimpleCurve {
    fn indices(&self) -> &[u32] {
        &self.indices
    }

    fn curve(&self) -> &[Point] {
        &self.points
    }
}

impl SimpleCurve {
    pub fn new() -> Self {
        Self {
            points: Vec::new(),
            indices: Vec::new(),
        }
    }

    pub fn register_2d_point(&mut self, point: Point2D) {
        self.points.push(Point::new(point.x, point.y, 0.0));
        self.update_indices();
    }

    pub fn register_3d_point(&mut self, point: Point) {
        self.points.push(point);
        self.update_indices();
    }

    pub fn register_2d_points(&mut self, points: &[Point2D]) {
        for point in points {
            self.points.push(Point::new(point.x, point.y, 0.0));
        }
        self.update_indices()
    }

    pub fn register_3d_points(&mut self, points: &[Point]) {
        for point in points {
            self.points.push(*point);
        }
        self.update_indices();
    }

    fn update_indices(&mut self) {
        self.indices = Vec::with_capacity(self.points.len() * 2);
        for i in 0..(self.points.len() as u32) {
            self.indices.push(i);
            if i < self.points.len() as u32 - 1 {
                self.indices.push(i + 1);
            }
        }
    }
}

impl From<&[Point]> for SimpleCurve {
    fn from(points: &[Point]) -> SimpleCurve {
        let mut curve = Self {
            points: points.to_vec(),
            indices: Vec::new(),
        };
        curve.update_indices();
        curve
    }
}

// A Cubic Bezier
#[derive(Clone)]
pub struct Bezier<const N: usize> {
    ctrl_points: [Point; N],
    curve_points: Vec<Point>,
    indices: Vec<u32>,
    nb_points: u32,
}

impl<const N: usize> Curve for Bezier<N> {
    fn curve(&self) -> &[Point] {
        &self.curve_points
    }

    fn indices(&self) -> &[u32] {
        &self.indices
    }
}

impl<const N: usize> Bezier<N> {
    pub fn new(begin: Point, end: Point, nb_segments: u32) -> Self {
        let mut ctrl_points = [Point::new(0.0, 0.0, 0.0); N];
        for i in 0..N {
            let d: f32 = i as f32 / (N - 1) as f32;
            ctrl_points[i] = begin * (1.0 - d) + end * d;
        }
        let mut bezier = Self {
            ctrl_points,
            curve_points: Vec::new(),
            indices: Vec::new(),
            nb_points: 2 * nb_segments,
        };
        bezier.evaluate();
        bezier
    }

    #[inline]
    pub fn ctrl_points(&self) -> &[Point] {
        &self.ctrl_points
    }

    pub fn ctrl_curve(&self) -> SimpleCurve {
        let mut curve = SimpleCurve::new();
        curve.register_3d_points(self.ctrl_points());
        curve
    }

    pub fn set_segments(&mut self, nb_segments: u32) {
        self.nb_points = 2 * nb_segments;
    }

    fn evaluate(&mut self) {
        self.curve_points = Vec::with_capacity(self.nb_points as usize);
        let mut u: f32 = 0.0;
        let epsilon = 1.0 / self.nb_points as f32;
        while u <= 1.0 {
            self.curve_points.push(de_casteljau(u, &self.ctrl_points));
            u += epsilon;
        }
        self.update_indices();
    }

    fn update_indices(&mut self) {
        let curve_size = self.curve_points.len();
        self.indices = Vec::with_capacity(curve_size * 2);
        for i in 0..(curve_size as u32 - 1) {
            self.indices.push(i);
            self.indices.push(i + 1);
        }
        self.indices.push(curve_size as u32 - 1);
    }
}

impl<const N: usize> From<[Point; N]> for Bezier<N> {
    fn from(points: [Point; N]) -> Bezier<N> {
        let mut bezier = Self {
            ctrl_points: points,
            curve_points: Vec::new(),
            indices: Vec::new(),
            nb_points: 100,
        };
        bezier.evaluate();
        bezier
    }
}

// TODO: PieceWiseBezier
pub struct PiecewiseBezier {
    ctrl_points: Vec<Point>,
    curve_points: Vec<Point>,
    indices: Vec<u32>,
    nb_points: u32,
}

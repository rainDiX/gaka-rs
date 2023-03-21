/*
* SPDX-License-Identifier: MIT
*/

pub mod curves;
pub mod mesh;
pub mod surfaces;

use nalgebra_glm as glm;

pub type Point = glm::Vec3;
pub type Point2D = glm::Vec2;

#[inline(always)]
fn de_casteljau(t: f32, points: &[Point]) -> Point {
    let mut points = points.to_vec();
    for i in 1..points.len() {
        for j in 0..(points.len() - i) {
            points[j] = points[j] * (1.0 - t) + points[j + 1] * t;
        }
    }
    points[0]
}

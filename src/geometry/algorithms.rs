/*
* SPDX-License-Identifier: MIT
*/

use super::Vec3;

pub fn de_casteljau(t: f32, points: &[Vec3]) -> Vec3 {
    let mut points = points.to_vec();
    for i in 1..points.len() {
        for j in 0..(points.len() - i) {
            points[j] = points[j] * (1.0 - t) + points[j + 1] * t;
        }
    }
    points[0]
}

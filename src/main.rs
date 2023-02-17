/*
* SPDX-License-Identifier: MIT
*/
mod rendering;
use rendering::glad2::draw_triangle;
// use rendering::glium::draw_triangle;

fn main() {
    draw_triangle();
}

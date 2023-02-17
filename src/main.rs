/*
* SPDX-License-Identifier: MIT
*/
mod rendering;
use rendering::gl_renderer::draw_triangle;
// use rendering::glium::draw_triangle;

fn main() {
    draw_triangle();
}

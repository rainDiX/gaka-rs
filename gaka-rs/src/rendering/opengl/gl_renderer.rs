/*
* SPDX-License-Identifier: MIT
*/

// use crate::rendering::vertex::Vertices;
use crate::{asset_manager::AssetManager, gl_check};

use glutin::prelude::GlDisplay;
use std::cell::{RefCell, RefMut};
use std::ffi::CString;
use std::ops::Deref;

// use super::gl_program::{ShaderProgram, ShaderType};
use super::gl_utils::show_platform_informations;
use super::gl_object::GlOject;

pub struct GlRenderer {
    objects: RefCell<Vec<GlOject>>,
}

impl GlRenderer {
    pub fn new<D: GlDisplay>(
        gl_display: &D
    ) -> Self {
        unsafe {
            gl::load_with(|symbol| {
                let symbol = CString::new(symbol).unwrap();
                gl_display.get_proc_address(symbol.as_c_str()).cast()
            });

            #[cfg(debug_assertions)]
            show_platform_informations();

            gl::Enable(gl::LINE_SMOOTH);
            Self { objects: RefCell::new(Vec::new()) }
        }
    }

    pub fn add_object(&mut self, object: GlOject) {
        self.objects.borrow_mut().push(object);
    }

    pub fn get_objects(&self) -> RefMut<Vec<GlOject>> {
        self.objects.borrow_mut()
    }

    pub fn draw(&self) {
        unsafe {
            gl_check!(gl::ClearColor(0.1, 0.1, 0.1, 1.0));
            gl_check!(gl::Clear(gl::COLOR_BUFFER_BIT));

            for object in self.objects.borrow().deref() {
                object.draw();
            }
        }
    }

    pub fn set_viewport(&self, x: i32, y: i32, width: i32, height: i32) {
        unsafe {
            gl_check!(gl::Viewport(x, y, width, height));
        }
    }

    pub fn resize(&self, width: i32, height: i32) {
        self.set_viewport(0, 0, width, height);
    }
}

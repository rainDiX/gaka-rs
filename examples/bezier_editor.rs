/*
* SPDX-License-Identifier: MIT
*/

use gaka_rs::asset_manager;
use gaka_rs::geometry;
use gaka_rs::geometry::curves::SimpleCurve;
use gaka_rs::rendering;

use asset_manager::AssetManager;
use gaka_rs::rendering::opengl::gl_object::DrawingMode;
use gaka_rs::rendering::opengl::gl_object::GlOject;
use gaka_rs::rendering::opengl::gl_program::ShaderProgram;
use gaka_rs::rendering::opengl::gl_program::ShaderType;
use gaka_rs::rendering::vertex::{VertexBuffer, Vertices};
use geometry::curves::{Bezier, Curve};

use winit::event::ElementState;
use winit::event::MouseButton;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoopBuilder;
use winit::window;
use winit::window::WindowBuilder;

use raw_window_handle::HasRawWindowHandle;

use glutin::config::ConfigTemplateBuilder;
use glutin::context::{ContextApi, ContextAttributesBuilder, GlProfile, Version};
use glutin::display::GetGlDisplay;
use glutin::prelude::*;
use glutin::surface::SwapInterval;

use glutin_winit::{self, DisplayBuilder, GlWindow};

use std::num::NonZeroU32;
use std::rc::Rc;

use nalgebra_glm as glm;

use glm::{Vec2, Vec3};

// use gl_renderer::GlRenderer;
use rendering::opengl::gl_renderer::GlRenderer;

fn main() {
    env_logger::init();

    let asset_manager = AssetManager::new("../gaka-rs/assets", true).unwrap();

    // create the window with glutin

    let (event_loop, mut window, gl_config) = {
        let el = EventLoopBuilder::new().build();
        let wb = WindowBuilder::new()
            .with_title("Bézier Editor Demo")
            .with_transparent(true)
            .with_inner_size(winit::dpi::LogicalSize::new(800.0, 600.0));

        let template = ConfigTemplateBuilder::new();

        let display_builder = DisplayBuilder::new().with_window_builder(Some(wb));

        let (window, gl_config) = display_builder
            .build(&el, template, |configs| {
                // Find the config with the maximum number of samples, so our triangle will
                // be smooth.
                configs
                    .reduce(|accum, config| {
                        let transparency_check = config.supports_transparency().unwrap_or(false)
                            & !accum.supports_transparency().unwrap_or(false);

                        if transparency_check || config.num_samples() > accum.num_samples() {
                            config
                        } else {
                            accum
                        }
                    })
                    .unwrap()
            })
            .unwrap();

        (el, window, gl_config)
    };

    let gl_display = gl_config.display();

    let raw_window_handle = window.as_ref().map(|window| window.raw_window_handle());

    let context_attributes = ContextAttributesBuilder::new()
        .with_context_api(ContextApi::OpenGl(Some(Version::new(4, 6))))
        .with_profile(GlProfile::Core)
        .build(raw_window_handle);

    let mut not_current_gl_context = Some(unsafe {
        gl_display
            .create_context(&gl_config, &context_attributes)
            .expect("Failed to create the OpenGL context")
    });

    let mut bezier = Bezier::new();

    let bezier_vertices = Vertices::from_curve(&mut bezier);
    let curve_vertices = Vertices::from_curve(&mut bezier.ctrl_curve());

    let mut state = None;
    let mut renderer = GlRenderer::new(&gl_display);
    let mut mouse_position = Vec2::new(0.0, 0.0);
    let mut winddow_size = Vec2::new(800.0, 600.0);

    event_loop.run(move |event, window_target, control_flow| {
        control_flow.set_wait();
        match event {
            Event::Resumed => {
                let window = window.take().unwrap_or_else(|| {
                    let window_builder = WindowBuilder::new().with_transparent(true);
                    glutin_winit::finalize_window(window_target, window_builder, &gl_config)
                        .unwrap()
                });

                let attrs = window.build_surface_attributes(<_>::default());
                let gl_surface = unsafe {
                    gl_config
                        .display()
                        .create_window_surface(&gl_config, &attrs)
                        .unwrap()
                };

                // Make it current.
                let gl_context = not_current_gl_context
                    .take()
                    .unwrap()
                    .make_current(&gl_surface)
                    .unwrap();

                let mut curve_program = ShaderProgram::new();
                curve_program
                    .compile_file("shaders/curve.vert", ShaderType::Vertex, &asset_manager)
                    .expect("Fail to compile File");
                curve_program
                    .compile_file("shaders/curve.frag", ShaderType::Fragment, &asset_manager)
                    .expect("Fail to compile File");
                curve_program.link().expect("Failed to Link Program");

                let curve_program = Rc::new(curve_program);
                let mut bezier_curve = GlOject::new(&curve_vertices, curve_program.clone());
                bezier_curve.set_drawing_mode(DrawingMode::Lines);
                let mut ctrl_curve = GlOject::new(&curve_vertices, curve_program.clone());
                ctrl_curve.set_drawing_mode(DrawingMode::Lines);
                renderer.add_object(bezier_curve);
                renderer.add_object(ctrl_curve);

                // Try setting vsync.
                if let Err(res) = gl_surface
                    .set_swap_interval(&gl_context, SwapInterval::Wait(NonZeroU32::new(1).unwrap()))
                {
                    log::error!("Error setting vsync: {:?}", res);
                }

                assert!(state.replace((gl_context, gl_surface, window)).is_none());
            }
            Event::RedrawRequested(_) => {
                if let Some((gl_context, gl_surface, _)) = &state {
                    renderer.draw();
                    // window.request_redraw();
                    gl_surface.swap_buffers(gl_context).unwrap();
                }
            }
            Event::WindowEvent { ref event, .. } => match event {
                WindowEvent::Resized(size) => {
                    if size.width != 0 && size.height != 0 {
                        // Some platforms like EGL require resizing GL surface to update the size
                        // Notable platforms here are Wayland and macOS, other don't require it
                        // and the function is no-op, but it's wise to resize it for portability
                        // reasons.
                        if let Some((gl_context, gl_surface, _)) = &state {
                            gl_surface.resize(
                                gl_context,
                                NonZeroU32::new(size.width).unwrap(),
                                NonZeroU32::new(size.height).unwrap(),
                            );
                        }
                        renderer.resize(size.width as i32, size.height as i32);
                        winddow_size.x = size.width as f32;
                        winddow_size.y = size.height as f32;
                    }
                }
                WindowEvent::CursorMoved {
                    device_id: _,
                    position,
                    ..
                } => {
                    mouse_position.x = 2. * position.x as f32 / winddow_size.x - 1.;
                    mouse_position.y = -(2. * position.y as f32 / winddow_size.y - 1.);
                }
                WindowEvent::MouseInput {
                    device_id: _,
                    state: button_state,
                    button,
                    ..
                } => match (button, button_state) {
                    (MouseButton::Left, ElementState::Pressed) => {
                        if let Some((_, _, window)) = &state {
                            bezier.register_point2d(mouse_position);
                            let mut objects = renderer.get_objects();
                            // objects[0].update(Vertices::from_curve(&mut bezier.ctrl_curve()).buffer);
                            objects[1].update(Vertices::from_curve(&mut bezier).buffer);
                            window.request_redraw();
                        }
                    }
                    _ => (),
                },
                WindowEvent::CloseRequested => control_flow.set_exit(),
                _ => (),
            },
            _ => (),
        }
    });
}

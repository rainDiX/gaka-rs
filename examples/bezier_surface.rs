/*
* SPDX-License-Identifier: MIT
*/

use gaka_rs::asset_manager;
use gaka_rs::geometry::curves::SimpleCurve;
use gaka_rs::geometry::Point;

use asset_manager::AssetManager;
use gaka_rs::geometry::surfaces::{BezierSurface, Surface};
use gaka_rs::rendering::Renderer;

use rand::Rng;
use winit::event::{
    ElementState, Event, MouseButton, MouseScrollDelta, VirtualKeyCode, WindowEvent,
};
use winit::event_loop::EventLoopBuilder;
use winit::window::WindowBuilder;

use raw_window_handle::HasRawWindowHandle;

use glutin::config::ConfigTemplateBuilder;
use glutin::context::{ContextApi, ContextAttributesBuilder, GlProfile, Version};
use glutin::display::GetGlDisplay;
use glutin::prelude::*;
use glutin::surface::SwapInterval;

use glutin_winit::{self, DisplayBuilder, GlWindow};

use std::num::NonZeroU32;

use nalgebra_glm as glm;

use glm::Vec2;

fn main() {
    env_logger::init();

    let asset_manager = AssetManager::new("../gaka-rs/assets", true).unwrap();

    // create the window with glutin

    let (event_loop, mut window, gl_config) = {
        let el = EventLoopBuilder::new().build();
        let wb = WindowBuilder::new()
            .with_title("Bézier Surfaces Demo")
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

    /* Create a random surface */
    let mut rng = rand::thread_rng();
    let mut ctrl_grid = [[Point::new(0.0, 0.0, 0.0); 4]; 4];
    for i in 0..4 {
        for j in 0..4 {
            ctrl_grid[i][j] = Point::new(i as f32, rng.gen::<f32>() * 4.0, j as f32);
        }
    }

    let surface = BezierSurface::new(ctrl_grid, 10);

    let mut curves = Vec::new();
    for i in 0..4 {
        let mut curve = SimpleCurve::new();
        curve.register_3d_points(&ctrl_grid[i]);
        curves.push(curve);
    }

    let mut state = None;
    let mut renderer = Renderer::new(&gl_display, asset_manager);
    let mut mouse_position = (-1.0, -1.0);
    let mut window_size = Vec2::new(800.0, 600.0);

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

                renderer.compile_shaders();

                let surface_mesh = renderer.create_object(surface.mesh());

                let mut curve_meshes = Vec::new();

                for curve in &curves {
                    curve_meshes.push(renderer.create_object(&curve.into()));
                }

                let scene = renderer.get_scene_mut();
                scene.add_object("surface", surface_mesh);

                let mut counter = 0;
                for curve in curve_meshes {
                    scene.add_object(&["curve", &counter.to_string()].join(""), curve);
                    counter += 1;
                }

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
                    renderer.render_scene();
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
                        window_size.x = size.width as f32;
                        window_size.y = size.height as f32;
                    }
                }
                WindowEvent::CursorMoved {
                    device_id: _,
                    position,
                    ..
                } => {
                    if mouse_position.0 > 0.0 {
                        if let Some((_, _, window)) = &state {
                            let camera = renderer.get_scene_mut().get_camera_mut();
                            camera.rotate_right(0.15 * (mouse_position.0 - position.x) as f32);
                            camera.rotate_up(0.15 * (mouse_position.1 - position.y) as f32);
                            window.request_redraw();
                        }
                    }

                    mouse_position.0 = position.x;
                    mouse_position.1 = position.y;
                }
                WindowEvent::MouseInput {
                    device_id: _,
                    state: button_state,
                    button,
                    ..
                } => match (button, button_state) {
                    (MouseButton::Left, ElementState::Pressed) => {
                        if let Some((_, _, window)) = &state {
                            let camera = renderer.get_scene_mut().get_camera_mut();
                            camera.move_by(10.0);
                            window.request_redraw();
                        }
                    }
                    _ => (),
                },
                WindowEvent::MouseWheel {
                    device_id: _,
                    delta,
                    ..
                } => {
                    if let Some((_, _, window)) = &state {
                        match delta {
                            MouseScrollDelta::LineDelta(_, vertical) => {
                                let camera = renderer.get_scene_mut().get_camera_mut();
                                camera.move_by(vertical * 0.5);
                                window.request_redraw();
                            }
                            _ => {}
                        }
                    }
                }
                WindowEvent::KeyboardInput {
                    device_id: _,
                    input,
                    ..
                } => match (input.virtual_keycode, input.state) {
                    (Some(VirtualKeyCode::Right), ElementState::Pressed) => {
                        if let Some((_, _, window)) = &state {
                            let camera = renderer.get_scene_mut().get_camera_mut();
                            camera.strafe_by(0.5);
                            window.request_redraw();
                        }
                    }
                    (Some(VirtualKeyCode::Left), ElementState::Pressed) => {
                        if let Some((_, _, window)) = &state {
                            let camera = renderer.get_scene_mut().get_camera_mut();
                            camera.strafe_by(-0.5);
                            window.request_redraw();
                        }
                    }
                    (Some(VirtualKeyCode::Up), ElementState::Pressed) => {
                        if let Some((_, _, window)) = &state {
                            let camera = renderer.get_scene_mut().get_camera_mut();
                            camera.move_up(0.5);
                            window.request_redraw();
                        }
                    }
                    (Some(VirtualKeyCode::Down), ElementState::Pressed) => {
                        if let Some((_, _, window)) = &state {
                            let camera = renderer.get_scene_mut().get_camera_mut();
                            camera.move_up(-0.5);
                            window.request_redraw();
                        }
                    }
                    _ => {}
                },
                WindowEvent::CloseRequested => control_flow.set_exit(),
                _ => (),
            },
            _ => (),
        }
    });
}

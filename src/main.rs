/*
* SPDX-License-Identifier: MIT
*/

extern crate glutin;
mod asset_manager;
mod rendering;

use asset_manager::AssetsManager;
use winit::event::{Event, WindowEvent};
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

use log::debug;
use log::error;
use log::info;
use log::warn;

// use gl_renderer::GlRenderer;
use rendering::opengl::gl_renderer::GlRenderer;

fn main() {

    env_logger::init();

    let asset_manager = AssetsManager::new("assets").unwrap();

    // create the window with glutin

    let (event_loop, mut window, gl_config) = {
        let el = EventLoopBuilder::new().build();
        let wb = WindowBuilder::new()
            .with_title("Demo")
            .with_transparent(true)
            .with_inner_size(winit::dpi::LogicalSize::new(800.0, 600.0));

        let template = ConfigTemplateBuilder::new();

        let display_builder = DisplayBuilder::new().with_window_builder(Some(wb));

        let (mut window, gl_config) = display_builder
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

    //     // link shaders
    //     let shader_program = gl::CreateProgram();
    //     gl::AttachShader(shader_program, vertex_shader);
    //     gl::AttachShader(shader_program, fragment_shader);
    //     gl::LinkProgram(shader_program);
    //     gl::GetProgramiv(shader_program, gl::LINK_STATUS, &mut success);
    //     if success == 0 {
    //         let mut info_log: Vec<i8> = Vec::with_capacity(512);
    //         let mut len = 0;
    //         gl::GetProgramInfoLog(
    //             shader_program,
    //             info_log.capacity() as gl::types::GLint,
    //             &mut len,
    //             info_log.as_mut_ptr(),
    //         );
    //         println!("ERROR::SHADER::PROGRAM::LINKING_FAILED");
    //         println!("{}", convert_info_log_to_string(&mut info_log, len));
    //     }
    // };

    let vertices: [f32; 18] = [
        // positions    // colors
        0.5, -0.5, 0.0, 1.0, 0.0, 0.0, // bottom right
        -0.5, -0.5, 0.0, 0.0, 1.0, 0.0, // bottom left
        0.0, 0.5, 0.0, 0.0, 0.0, 1.0, // top
    ];

    let mut state = None;
    let mut renderer = None;

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

                renderer.get_or_insert_with(|| {
                    GlRenderer::new(&gl_display, &asset_manager, &vertices)
                });

                // Try setting vsync.
                if let Err(res) = gl_surface
                    .set_swap_interval(&gl_context, SwapInterval::Wait(NonZeroU32::new(1).unwrap()))
                {
                    error!("Error setting vsync: {:?}", res);
                }

                assert!(state.replace((gl_context, gl_surface, window)).is_none());
            }
            Event::RedrawRequested(_) => {
                if let Some((gl_context, gl_surface, _)) = &state {
                    let renderer = renderer.as_ref().unwrap();
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
                        let renderer = renderer.as_ref().unwrap();
                        renderer.resize(size.width as i32, size.height as i32);
                    }
                }
                WindowEvent::CloseRequested => control_flow.set_exit(),
                _ => (),
            },
            _ => (),
        }
    });
}

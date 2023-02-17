pub fn create_window() {

}

extern crate glad_gl;
extern crate glutin;

use glad_gl::gl;

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

use std::ffi::{self, CString};
use std::fs;
use std::num::NonZeroU32;

pub fn draw_triangle() {
    // create the window with glutin

    let (event_loop, mut window, gl_config) = {
        let el = EventLoopBuilder::new().build();
        let wb = WindowBuilder::new()
            .with_title("Hello world!")
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

    // load OpenGL functions
    let gl = gl::load(|addr| {
        let addr = CString::new(addr).unwrap();
        gl_display.get_proc_address(addr.as_c_str()).cast()
    });

    let mut not_current_gl_context = Some(unsafe {
        gl_display
            .create_context(&gl_config, &context_attributes)
            .expect("Failed to create the OpenGL context")
    });
 
    let mut state = None;

    event_loop.run(move |event, window_target, control_flow| {
        control_flow.set_wait();
        match event {
            Event::Resumed => {
                #[cfg(target_os = "android")]
                println!("Android window available");

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

                //

                // Try setting vsync.
                if let Err(res) = gl_surface
                    .set_swap_interval(&gl_context, SwapInterval::Wait(NonZeroU32::new(1).unwrap()))
                {
                    eprintln!("Error setting vsync: {:?}", res);
                }

                assert!(state.replace((gl_context, gl_surface, window)).is_none());
            }
            Event::LoopDestroyed => {
                return;
            }
            // Event::MainEventsCleared => {
            //     windowed_context.window().request_redraw();
            // }
            Event::RedrawRequested(_) => {
                unsafe {
                    gl::ClearColor(0.2, 0.3, 0.3, 1.0);
                    gl::Clear(gl::COLOR_BUFFER_BIT);
                    // use the program
                    gl::UseProgram(shader_program);
                    gl::BindVertexArray(vao);
                    // draw our triangle
                    gl::DrawArrays(gl::TRIANGLES, 0, 3)
                }
                if let Some((gl_context, gl_surface, _)) = &state {
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
                        unsafe {
                            gl::Viewport(0, 0, size.width as i32, size.height as i32);
                        }
                    }
                }
                WindowEvent::CloseRequested => {
                    unsafe {
                        gl::DeleteVertexArrays(1, &mut vao as *mut u32);
                        gl::DeleteBuffers(1, &mut vbo as *mut u32);
                        gl::DeleteProgram(shader_program);
                    }
                    control_flow.set_exit()
                }
                _ => (),
            },
            _ => (),
        }
    });
}

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

    //  Shaders source code
    let vertex_shader =
        fs::read_to_string("./glsl/vertexShader.vert").expect("fail to read the file");
    let fragment_shader =
        fs::read_to_string("./glsl/fragmentShader.frag").expect("fail to read the file");
    let v_shader_code = ffi::CString::new(vertex_shader).unwrap();
    let f_shader_code = ffi::CString::new(fragment_shader).unwrap();

    // build and compile the shader program
    let mut success: gl::types::GLint = 1;
    let shader_program: u32 = unsafe {
        // vertex shader
        let vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);
        gl::ShaderSource(vertex_shader, 1, &v_shader_code.as_ptr(), std::ptr::null());
        gl::CompileShader(vertex_shader);
        gl::GetShaderiv(vertex_shader, gl::COMPILE_STATUS, &mut success);
        if success == 0 {
            let mut info_log: Vec<i8> = Vec::with_capacity(512);
            let mut len = 0;
            gl::GetShaderInfoLog(
                vertex_shader,
                info_log.capacity() as gl::types::GLint,
                &mut len,
                info_log.as_mut_ptr(),
            );
            println!("ERROR::SHADER::VERTEX::COMPILATION_FAILED");
            println!("{}", convert_info_log_to_string(&mut info_log, len));
        }
        // fragment shader
        let fragment_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
        gl::ShaderSource(
            fragment_shader,
            1,
            &f_shader_code.as_ptr(),
            std::ptr::null(),
        );
        gl::CompileShader(fragment_shader);
        gl::GetShaderiv(fragment_shader, gl::COMPILE_STATUS, &mut success);
        if success == 0 {
            let mut info_log: Vec<i8> = Vec::with_capacity(512);
            let mut len = 0;
            gl::GetShaderInfoLog(
                fragment_shader,
                info_log.capacity() as gl::types::GLint,
                &mut len,
                info_log.as_mut_ptr(),
            );
            println!("ERROR::SHADER::FRAGMENT::COMPILATION_FAILED");
            println!("{}", convert_info_log_to_string(&mut info_log, len));
        }

        // link shaders
        let shader_program = gl::CreateProgram();
        gl::AttachShader(shader_program, vertex_shader);
        gl::AttachShader(shader_program, fragment_shader);
        gl::LinkProgram(shader_program);
        gl::GetProgramiv(shader_program, gl::LINK_STATUS, &mut success);
        if success == 0 {
            let mut info_log: Vec<i8> = Vec::with_capacity(512);
            let mut len = 0;
            gl::GetProgramInfoLog(
                shader_program,
                info_log.capacity() as gl::types::GLint,
                &mut len,
                info_log.as_mut_ptr(),
            );
            println!("ERROR::SHADER::PROGRAM::LINKING_FAILED");
            println!("{}", convert_info_log_to_string(&mut info_log, len));
        }
        gl::DeleteShader(vertex_shader);
        gl::DeleteShader(fragment_shader);
        shader_program
    };

    let vertices: [f32; 9] = [
        -0.5, -0.5, 0.0, // left
        0.5, -0.5, 0.0, // right
        0.0, 0.5, 0.0, // top
    ];

    let mut vao: u32 = 0;
    let mut vbo: u32 = 0;

    unsafe {
        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut vbo);
        gl::BindVertexArray(vao);

        // copy the vertices array in a buffer
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (vertices.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
            vertices.as_ptr().cast(),
            gl::STATIC_DRAW,
        );
        // set the vertex attributes pointers
        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            (3 * std::mem::size_of::<f32>()) as gl::types::GLsizei,
            std::ptr::null(),
        );
        gl::EnableVertexAttribArray(0);
    }

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

fn convert_info_log_to_string(info_log: &mut Vec<i8>, len: i32) -> String {
    let log = unsafe {
        info_log.set_len(len as usize);
        std::slice::from_raw_parts(info_log.as_ptr() as *const u8, info_log.len())
    };
    String::from_utf8(log.to_vec()).expect("Found invalid UTF-8")
}

pub struct Renderer {
    program: gl::types::GLuint,
    vao: gl::types::GLuint,
    vbo: gl::types::GLuint,
}
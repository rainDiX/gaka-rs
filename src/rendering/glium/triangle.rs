extern crate glium;

use std::fs;
use glium::{glutin, implement_vertex, Surface};

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 3],
}

pub fn draw_triangle() {
    // create the glutin window
    let event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new();
    let cb = glutin::ContextBuilder::new();
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();

    implement_vertex!(Vertex, position);

    let triangle = vec![
        Vertex {
            position: [-0.5, -0.5, 0.0],
        },
        Vertex {
            position: [0.5, -0.5, 0.0],
        },
        Vertex {
            position: [0.0, 0.5, 0.0],
        }
    ];

    let vb = glium::VertexBuffer::new(&display, &triangle).unwrap();
    let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

    let vertex_shader = fs::read_to_string("./glsl/vertexShader.vert").expect("fail to read the file");
    let fragment_shader = fs::read_to_string("./glsl/fragmentShader.frag").expect("fail to read the file");

    let source = glium::program::ProgramCreationInput::SourceCode {
        vertex_shader: &vertex_shader,
        fragment_shader: &fragment_shader,
        geometry_shader: None,
        outputs_srgb: true,
        uses_point_size: false,
        tessellation_control_shader: None,
        tessellation_evaluation_shader: None,
        transform_feedback_varyings: None,
    };

    let program = match glium::Program::new(&display, source) {
        Ok(p) => p,
        Err(e) => panic!("{:?}", e),
    };

    event_loop.run(move |event, _, control_flow| {
        *control_flow = glutin::event_loop::ControlFlow::Wait;
        match event {
            glutin::event::Event::LoopDestroyed => {
                return;
            }
            glutin::event::Event::RedrawRequested(_) => {
                let mut target = display.draw();
                target.clear_color(0.2, 0.3, 0.3, 1.0);
                target
                    .draw(
                        &vb,
                        &indices,
                        &program,
                        &glium::uniforms::EmptyUniforms,
                        &Default::default(),
                    )
                    .unwrap();
                target.finish().unwrap();
            }
            glutin::event::Event::WindowEvent { ref event, .. } => match event {
                glutin::event::WindowEvent::CloseRequested => {
                    *control_flow = glutin::event_loop::ControlFlow::Exit
                }
                _ => (),
            },
            _ => (),
        }
    });
}

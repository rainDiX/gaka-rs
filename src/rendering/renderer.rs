use glutin::prelude::GlDisplay;

pub trait Renderer {
    fn new<D: GlDisplay>(
        gl_display: &D,
        vertex_shader_source: &String,
        fragment_shader_source: &String,
        vertex_data: &[f32],
    ) -> Self;

    fn draw(&self);

    fn set_viewport(&self, x: i32, y: i32, width:i32, height:i32);

    fn resize(&self, width: i32, height: i32);
}

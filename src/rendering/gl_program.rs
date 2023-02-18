mod gl {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub struct GLProgram {
    program: gl::types::GLuint,
    gl: gl::Gl,
}
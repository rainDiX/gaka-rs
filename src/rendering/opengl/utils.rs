extern crate gl;

#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! gl_check {
    ($fun: expr) => {{
        $fun
    }};
}

#[cfg(debug_assertions)]
#[macro_export]
macro_rules! gl_check {
    ($fun:expr) => {{
        let (val, err) = ($fun, gl::GetError());
        if err != gl::NO_ERROR {
            let error: String = match err {
                gl::INVALID_ENUM => "GL_INVALID_ENUM".into(),
                gl::INVALID_VALUE => "GL_INVALID_VALUE".into(),
                gl::INVALID_OPERATION => "GL_INVALID_OPERATION".into(),
                gl::STACK_OVERFLOW => "GL_STACK_OVERFLOW".into(),
                gl::STACK_UNDERFLOW => "GL_STACK_UNDERFLOW".into(),
                gl::OUT_OF_MEMORY => "GL_OUT_OF_MEMORY".into(),
                gl::INVALID_FRAMEBUFFER_OPERATION => "GL_INVALID_FRAMEBUFFER_OPERATION".into(),
                gl::CONTEXT_LOST => "GL_CONTEXT_LOST".into(),
                _ => format!("GL_?_{}", err).into(),
            };
            log::error!("OpenGL error {} in {}", error, stringify!($fun));
        }
        val
    }};
}

#[cfg(not(debug_assertions))]
pub unsafe fn gl_comp_status(shader: gl::types::GLuint) {}

#[cfg(debug_assertions)]
pub unsafe fn gl_comp_status(shader: gl::types::GLuint) {
    let mut success: gl::types::GLint = 0;
    let mut len: gl::types::GLint = 0;
    gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
    gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);

    if success == 0 {
        let mut info_log: Vec<i8> = Vec::with_capacity(len as usize + 1);
        gl::GetShaderInfoLog(shader, len, std::ptr::null_mut(), info_log.as_mut_ptr());
        eprintln!("ERROR SHADER COMPILATION FAILED");
        eprintln!("{}", convert_info_log_to_string(&mut info_log, len));
    }
}


#[cfg(not(debug_assertions))]
pub unsafe fn gl_link_status(shader: gl::types::GLuint) {}

#[cfg(debug_assertions)]
pub unsafe fn gl_link_status(program: gl::types::GLuint) {
    let mut success: gl::types::GLint = 0;
    let mut len: gl::types::GLint = 0;
    gl::GetProgramiv(program, gl::LINK_STATUS, &mut success);
    gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
    if success == 0 {
        let mut info_log: Vec<i8> = Vec::with_capacity(len as usize + 1);
        gl::GetProgramInfoLog(program, len, std::ptr::null_mut(), info_log.as_mut_ptr());
        println!("ERROR PROGRAM LINKING_FAILED");
        println!("{}", convert_info_log_to_string(&mut info_log, len));
    }
}

#[cfg(debug_assertions)]
fn convert_info_log_to_string(info_log: &mut Vec<i8>, len: i32) -> String {
    let log = unsafe {
        info_log.set_len(len as usize);
        std::slice::from_raw_parts(info_log.as_ptr() as *const u8, info_log.len())
    };
    String::from_utf8(log.to_vec()).expect("Found invalid UTF-8")
}

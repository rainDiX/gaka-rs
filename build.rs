use std::{fs, path};

fn build_hlsl_shader(file_path: &str, options: &shaderc::CompileOptions) {
    let compiler = shaderc::Compiler::new().unwrap();
    let shader_source =
        fs::read_to_string(file_path).unwrap_or_else(|_| panic!("failed to read {}", file_path));

    let vertex_shader_entry = "VSMain";
    let fragment_shader_entry = "PSMain";

    let out = file_path
        .split('/')
        .last()
        .unwrap()
        .split_once('.')
        .unwrap()
        .0;

    let binary_result = compiler
        .compile_into_spirv(
            &shader_source,
            shaderc::ShaderKind::Vertex,
            file_path,
            vertex_shader_entry,
            Some(options),
        )
        .unwrap();

    let out_path = format!("assets/spirv/{}.vert.spv", out);
    fs::write(out_path, binary_result.as_binary_u8()).unwrap();

    let text_result = compiler
        .compile_into_spirv_assembly(
            &shader_source,
            shaderc::ShaderKind::Vertex,
            file_path,
            vertex_shader_entry,
            Some(options),
        )
        .unwrap();

    let out_path = format!("assets/spirv/{}.vert.spvasm", out);
    std::fs::write(out_path, text_result.as_text()).unwrap();

    let binary_result = compiler
        .compile_into_spirv(
            &shader_source,
            shaderc::ShaderKind::Fragment,
            file_path,
            fragment_shader_entry,
            Some(options),
        )
        .unwrap();

    let out_path = format!("assets/spirv/{}.frag.spv", out);
    fs::write(out_path, binary_result.as_binary_u8()).unwrap();

    let text_result = compiler
        .compile_into_spirv_assembly(
            &shader_source,
            shaderc::ShaderKind::Fragment,
            file_path,
            fragment_shader_entry,
            Some(options),
        )
        .unwrap();

    let out_path = format!("assets/spirv/{}.frag.spvasm", out);
    std::fs::write(out_path, text_result.as_text()).unwrap();
}

// Example custom build script.
fn main() {
    let mut options = shaderc::CompileOptions::new().unwrap();
    options.set_source_language(shaderc::SourceLanguage::HLSL);

    println!("cargo::rerun-if-changed=shaders/basic.hlsl");
    let hlsl_shaders = ["shaders/basic.hlsl"];

    if path::Path::new("assets/spirv").exists() {
        fs::remove_dir_all("assets/spirv").expect("Failed to recreate oupput dir");
    }
    fs::create_dir_all("assets/spirv").expect("Failed to create output dir");

    hlsl_shaders
        .iter()
        .for_each(|s| build_hlsl_shader(s, &options));
}

#version 460 core

layout (location = 0) in vec3 position;
layout (location = 1) in vec3 normal;
layout (location = 2) in vec2 tex_coords;

out vec3 vs_normal;
out vec2 vs_tex_coords;

// uniform mat4 model;
// uniform mat4 view;
// uniform mat4 projection;

void main()
{
    // note that we read the multiplication from right to left
   //  gl_Position = projection * view * model * position;
   gl_Position = vec4(position, 1.0);
   vs_normal = normal;
   vs_tex_coords = tex_coords;
}

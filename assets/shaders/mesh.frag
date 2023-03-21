#version 460 core
out vec4 color;

in vec3 vs_normal;
in vec2 vs_tex_coords;

void main()
{
   color = vec4(vs_normal.x * vs_tex_coords.x, vs_normal.y * vs_tex_coords.y, vs_normal.z, 1.0);
}
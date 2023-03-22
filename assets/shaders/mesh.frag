#version 460 core
out vec4 color;

in vec3 vs_normal;
in vec2 vs_tex_coords;

void main()
{
   color = vec4(0.5 * vs_normal.x * vs_tex_coords.x + 0.5, 0.5 * vs_normal.y * vs_tex_coords.y + 0.5, 0.5 * vs_normal.z + 0.5, 1.0);
}
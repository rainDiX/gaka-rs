#version 460 core
out vec4 color;
in vec3 vs_color;

void main()
{
   color = vec4(vs_color, 1.0f);
}
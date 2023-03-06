#version 460 core
layout (location = 0) in vec4 position;

void main()
{
   gl_Position = vec4(position.x, position.y, position.z, position.w);
}

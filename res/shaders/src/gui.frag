#version 450

layout(set = 0, binding = 0) uniform sampler2D tex;

layout(location = 0) in vec4 col;
layout(location = 1) in vec2 uv;

layout(location = 0) out vec4 out_col;

void main() 
{
    //out_col = vec4(col);
	out_col = col * texture(tex, uv);
}
#version 450

layout(location = 0) in vec3 norm;

layout(location = 0) out vec4 out_col;

void main() 
{
	out_col = vec4(norm * 0.5 + 0.5, 1.0);
}

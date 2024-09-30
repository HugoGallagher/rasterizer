#version 450

layout(push_constant) uniform push_constants {
    mat4 view_proj;
} mats;

layout(std140, set=0, binding=0) readonly buffer MeshData {
    mat4 transforms[];
} mesh_data;

layout(location = 0) in vec3 pos;
layout(location = 1) in vec3 norm;

layout(location = 0) out vec3 f_norm;

void main() {
    gl_Position = mats.view_proj * mesh_data.transforms[gl_InstanceIndex] * vec4(pos, 1);
    f_norm = norm;
}
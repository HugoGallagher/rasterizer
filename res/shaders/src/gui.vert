#version 450

layout(location = 0) in vec2 pos;
layout(location = 1) in vec2 uv;
layout(location = 2) in vec4 col;

layout(push_constant) uniform push_constnat {
    vec2 scale;
    vec2 pre_translate;
} pc;

layout(location = 0) out vec4 f_col;
layout(location = 1) out vec2 f_uv;

void main() {
    gl_Position = vec4((pos + pc.pre_translate) * pc.scale, 0.5, 1.0);
    f_col = col;
    f_uv = uv;
}
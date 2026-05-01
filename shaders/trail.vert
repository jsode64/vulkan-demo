#version 450

struct Vertex {
    vec4 c;
    vec2 p;
    vec2 v;
    float m;
};

layout(std430, binding = 1) buffer VertexBuffer {
    Vertex vertices[];
};

layout(location = 0) out vec4 outColor;

// The bound. X and Y values are limited to [-bound, bound].
const float BOUND = 1000.0;

void main() {
    const float PI = 3.14159265359;
    const Vertex me = vertices[gl_InstanceIndex];
    const vec2 start = (me.p - me.v) / BOUND;
    const vec2 end = me.p / BOUND;
    const float dir = atan(me.v.y, me.v.x);
    const float left = dir + (PI / 2.0);
    const float right = dir - (PI / 2.0);
    const float size = sqrt(me.m) * 0.0005;

    if (gl_VertexIndex == 0) {
        gl_Position = vec4(start + (size * vec2(cos(left), sin(left))), 0.0, 1.0);
    } else if (gl_VertexIndex == 1) {
        gl_Position = vec4(end + (size * vec2(cos(left), sin(left))), 0.0, 1.0);
    } else if (gl_VertexIndex == 2) {
        gl_Position = vec4(start + (size * vec2(cos(right), sin(right))), 0.0, 1.0);
    } else {
        gl_Position = vec4(end + (size * vec2(cos(right), sin(right))), 0.0, 1.0);
    }
    
    outColor = me.c;
}

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
    const float dir = atan(me.v.y, me.v.x);
    const float size = sqrt(me.m) * 0.0005;


    float angle = 0.0;
    if (gl_VertexIndex == 0) {
        angle = dir;
    } else if (gl_VertexIndex == 1) {
        angle = dir + (2.0 * PI / 3.0);
    } else {
        angle = dir - (2.0 * PI / 3.0);
    }
    
    gl_Position = vec4((me.p / BOUND) + (size * vec2(cos(angle), sin(angle))), 0.0, 1.0);
    outColor = me.c;
}

#version 440

layout(location = 0) in vec2 qt_TexCoord0;
layout(location = 0) out vec4 frag_color;

layout(std140, binding = 0) uniform buf {
    mat4 qt_Matrix;
    float qt_Opacity;

    float progress;
};
layout(binding = 1) uniform sampler2D waveform;

#define uv qt_TexCoord0

void main() {
    vec4 wave_data = texture(waveform, vec2(uv.x, 0.0)) * 1.5;

    float inside = clamp(sign(wave_data.a - (1.0 - uv.y)), 0.0, 1.0);

    float inside_high = clamp(sign(wave_data.a - (1.0 - uv.y)), 0.0, 1.0);
    float inside_mid = clamp(sign((wave_data.a - wave_data.b) - (1.0 - uv.y)), 0.0, 1.0);
    float inside_low = clamp(sign((wave_data.a - (wave_data.b + wave_data.g)) - (1.0 - uv.y)), 0.0, 1.0);

    frag_color = vec4(0.0);
    frag_color = vec4(1.0, 1.0, 1.0, 1.0) * (inside_high);
    frag_color = mix(vec4(0.996978431373, 0.607843137255, 0.16862745098, 1.0), frag_color, 1.0 - inside_mid);
    frag_color = mix(vec4(0.0745098039216, 0.305882352941, 0.843137254902, 1.0), frag_color, 1.0 - inside_low);

    float has_passed = clamp(sign(progress - uv.x), 0.0, 1.0);

    frag_color = frag_color * inside * mix(vec4(1.0, 1.0, 1.0, 1.0), vec4(0.5, 0.5, 0.5, 1.0), has_passed);
}

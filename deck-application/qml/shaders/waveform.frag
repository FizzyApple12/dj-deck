#version 440

layout(location = 0) in vec2 qt_TexCoord0;
layout(location = 0) out vec4 frag_color;

layout(std140, binding = 0) uniform buf {
    mat4 qt_Matrix;
    float qt_Opacity;

    float stride;
};
layout(binding = 1) uniform sampler2D waveform;

#define uv qt_TexCoord0

void main() {
    vec2 target_position = vec2(
            mod(uv.x * stride, 1.0),
            floor(uv.x * stride) / ceil(stride)
        );
    vec4 wave_data = texture(waveform, target_position);

    float inside = clamp(sign(wave_data.a - abs(0.5 - uv.y)), 0.0, 1.0);

    float inside_high = clamp(sign(wave_data.r - abs(0.5 - uv.y)), 0.0, 1.0);
    float inside_mid = clamp(sign(wave_data.g - abs(0.5 - uv.y)), 0.0, 1.0);
    float inside_low = clamp(sign(wave_data.b - abs(0.5 - uv.y)), 0.0, 1.0);

    // frag_color = vec4(0.0);
    // frag_color = mix(
    //         vec4(0.0745098039216, 0.305882352941, 0.843137254902, 1.0),
    //         frag_color,
    //         1.0 - inside_low
    //     );
    // frag_color = mix(
    //         vec4(0.996978431373, 0.607843137255, 0.16862745098, 1.0),
    //         frag_color,
    //         1.0 - inside_mid
    //     );
    // frag_color = mix(
    //         vec4(0.662745098039, 0.360784313725, 0.117647058824, 1.0),
    //         frag_color,
    //         1.0 - min(inside_low, inside_mid)
    //     );
    // frag_color = mix(vec4(1.0, 1.0, 1.0, 1.0), frag_color, 1.0 - inside_high);

    frag_color = vec4(0.0);
    frag_color = mix(vec4(vec3(0.0), 1.0), frag_color, 1.0 - inside_high);
    frag_color = mix(
            vec4(0.0745098039216, 0.305882352941, 0.843137254902, 1.0),
            frag_color,
            1.0 - inside_low
        );
    frag_color = mix(
            vec4(0.996978431373, 0.607843137255, 0.16862745098, 1.0),
            frag_color,
            1.0 - inside_mid
        );
    frag_color = mix(
            vec4(0.662745098039, 0.360784313725, 0.117647058824, 1.0),
            frag_color,
            1.0 - min(inside_low, inside_mid)
        );
    frag_color = mix(frag_color * vec4(0.3) + vec4(0.7), frag_color, 1.0 - inside_high);

    frag_color = min(frag_color, vec4(1.0));
}

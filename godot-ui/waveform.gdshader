shader_type canvas_item;

uniform sampler2D WAVEFORM;
uniform float STRIDE;

void vertex() {
	
}

void fragment() {
  vec2 target_position = vec2(mod(UV.x * STRIDE, 1.0), floor(UV.x * STRIDE) / (ceil(STRIDE)));
  vec4 wave_data = texture(WAVEFORM, target_position);

  float inside = clamp(sign(wave_data.a - abs(0.5 - UV.y)), 0.0, 1.0);

  float inside_high = clamp(sign(wave_data.r - abs(0.5 - UV.y)), 0.0, 1.0);
  float inside_mid = clamp(sign(wave_data.g - abs(0.5 - UV.y)), 0.0, 1.0);
  float inside_low = clamp(sign(wave_data.b - abs(0.5 - UV.y)), 0.0, 1.0);

  //COLOR = vec4(0.0);
  //COLOR = mix(vec4(0.0745098039216, 0.305882352941, 0.843137254902, 1.0), COLOR, 1.0 - inside_low);
  //COLOR = mix(vec4(0.996978431373, 0.607843137255, 0.16862745098, 1.0), COLOR, 1.0 - inside_mid);
  //COLOR = mix(vec4(0.662745098039, 0.360784313725, 0.117647058824, 1.0), COLOR, 1.0 - min(inside_low, inside_mid));
  //COLOR = mix(vec4(1.0, 1.0, 1.0, 1.0), COLOR, 1.0 - inside_high);

  COLOR = vec4(0.0);
  COLOR = mix(vec4(vec3(0.0), 1.0), COLOR, 1.0 - inside_high);
  COLOR = mix(vec4(0.0745098039216, 0.305882352941, 0.843137254902, 1.0), COLOR, 1.0 - inside_low);
  COLOR = mix(vec4(0.996978431373, 0.607843137255, 0.16862745098, 1.0), COLOR, 1.0 - inside_mid);
  COLOR = mix(vec4(0.662745098039, 0.360784313725, 0.117647058824, 1.0), COLOR, 1.0 - min(inside_low, inside_mid));
  COLOR = mix(COLOR * vec4(0.3) + vec4(0.7), COLOR, 1.0 - inside_high);

  COLOR = min(COLOR, vec4(1.0));
}

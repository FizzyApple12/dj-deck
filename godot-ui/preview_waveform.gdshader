shader_type canvas_item;

uniform sampler2D WAVEFORM;
uniform float PROGRESS;

void vertex() {
	
}

void fragment() {
  vec4 wave_data = texture(WAVEFORM, vec2(UV.x, 0.0)) * 1.5;

  float inside = clamp(sign(wave_data.a - (1.0 - UV.y)), 0.0, 1.0);

  float inside_high = clamp(sign(wave_data.a - (1.0 - UV.y)), 0.0, 1.0);
  float inside_mid = clamp(sign((wave_data.a - wave_data.b) - (1.0 - UV.y)), 0.0, 1.0);
  float inside_low = clamp(sign((wave_data.a - (wave_data.b + wave_data.g)) - (1.0 - UV.y)), 0.0, 1.0);

  COLOR = vec4(0.0);
  COLOR = vec4(1.0, 1.0, 1.0, 1.0) * (inside_high);
  COLOR = mix(vec4(0.996978431373, 0.607843137255, 0.16862745098, 1.0), COLOR, 1.0 - inside_mid);
  COLOR = mix(vec4(0.0745098039216, 0.305882352941, 0.843137254902, 1.0), COLOR, 1.0 - inside_low);

  float has_passed = clamp(sign(PROGRESS - UV.x), 0.0, 1.0);

  COLOR = COLOR * inside * mix(vec4(1.0, 1.0, 1.0, 1.0), vec4(0.5, 0.5, 0.5, 1.0), has_passed);
}

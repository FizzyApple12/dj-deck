shader_type canvas_item;

uniform sampler2D WAVEFORM;
uniform float STRIDE;

void vertex() {
	
}

void fragment() {
  vec2 target_position = vec2(mod(UV.x * STRIDE, 1.0), floor(UV.x * STRIDE) / (ceil(STRIDE)));
  vec4 wave_data = texture(WAVEFORM, target_position);
  //wave_data = vec4(pow(wave_data.r, 2.0), pow(wave_data.g, 2.0), pow(wave_data.b, 2.0),pow(wave_data.a, 2.0));

  float inside = clamp(sign(wave_data.a - abs(0.5 - UV.y)), 0.0, 1.0);

  float inside_high = clamp(sign(wave_data.r - abs(0.5 - UV.y)), 0.0, 1.0);
  float inside_mid = clamp(sign(wave_data.g - abs(0.5 - UV.y)), 0.0, 1.0);
  float inside_low = clamp(sign(wave_data.b - abs(0.5- UV.y)), 0.0, 1.0);

  float inside_low_mid = min(inside_low, inside_mid);

  COLOR = vec4(0.0);
  COLOR = mix(vec4(0.0745098039216, 0.305882352941, 0.843137254902, 1.0), COLOR, 1.0 - inside_low);
  COLOR = mix(vec4(0.996978431373, 0.607843137255, 0.16862745098, 1.0), COLOR, 1.0 - inside_mid);
  COLOR = mix(vec4(0.662745098039, 00.360784313725, 0.117647058824, 1.0), COLOR, 1.0 - inside_low_mid);
  COLOR = mix(vec4(1.0, 1.0, 1.0, 1.0), COLOR, 1.0 - inside_high);

  COLOR = COLOR * inside;
}

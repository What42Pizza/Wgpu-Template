@group(0) @binding(0) var<uniform> color_correction_data: ColorCorrectionData;

struct ColorCorrectionData {
	saturation: f32,
	brightness: f32,
}



@vertex
fn vs_main(
	@builtin(vertex_index) index: u32
) -> VertexOutput {
	var output: VertexOutput;
	
	// hacky way to draw a single large triangle that convers the entire screen
	output.screen_pos = vec4(
		f32(i32(index) / 2) * 4.0 - 1.0,
		f32(i32(index) & 1) * 4.0 - 1.0,
		1.0,
		1.0,
	);
	
	output.tex_coords = output.screen_pos.xy * vec2(0.5, -0.5) + 0.5;
	
	return output;
}



struct VertexOutput {
	@builtin(position) screen_pos: vec4f,
	@location(0) tex_coords: vec2f,
}

@group(0) @binding(1) var main_texture: texture_2d<f32>;
@group(0) @binding(2) var main_sampler: sampler;



@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
	var color = textureSample(main_texture, main_sampler, in.tex_coords).rgb;
	
	// saturation
	let color_lum = dot(color, vec3(0.2125, 0.7154, 0.0721));
	color = mix(vec3(color_lum), color, color_correction_data.saturation);
	
	// brightness
	color *= color_correction_data.brightness;
	
	return vec4(color, 1.0);
}

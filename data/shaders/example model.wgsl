@group(0) @binding(0) var<uniform> camera_data: CameraData;
@group(1) @binding(0) var<uniform> shadowmap_proj_mat: mat4x4<f32>;

struct CameraData {
	proj_view_mat: mat4x4<f32>,
	inv_proj_mat: mat4x4<f32>,
	view_mat: mat4x4<f32>,
}

struct BasicVertexInput {
	@location(0) position: vec3<f32>,
}

struct ExtendedVertexInput {
	@location(1) texcoords: vec2<f32>,
	@location(2) normal: vec2<f32>,
}

struct InstanceInput {
	@location(3) model_mat_0: vec4<f32>,
	@location(4) model_mat_1: vec4<f32>,
	@location(5) model_mat_2: vec4<f32>,
	@location(6) model_mat_3: vec4<f32>,
};



@vertex
fn vs_main(
	vertex_basic: BasicVertexInput,
	vertex_extended: ExtendedVertexInput,
	instance: InstanceInput,
) -> VertexOutput {
	
	let instance_mat = mat4x4<f32>(
		instance.model_mat_0,
		instance.model_mat_1,
		instance.model_mat_2,
		instance.model_mat_3,
	);
	
	let world_pos = instance_mat * vec4<f32>(vertex_basic.position, 1.0);
	
	var out: VertexOutput;
	out.screen_pos = camera_data.proj_view_mat * instance_mat * vec4<f32>(vertex_basic.position, 1.0);
	out.screen_pos.z = out.screen_pos.z * 0.5 + 0.25;
	out.world_pos = world_pos.xyz;
	out.texcoords = vertex_extended.texcoords;
	return out;
}



struct VertexOutput {
	@builtin(position) screen_pos: vec4<f32>,
	@location(0) world_pos: vec3<f32>,
	@location(1) texcoords: vec2<f32>,
};

@group(2) @binding(0) var material_texture: texture_2d<f32>;
@group(2) @binding(1) var material_sampler: sampler;

@group(3) @binding(0) var shadowmap_texture: texture_depth_2d;
@group(3) @binding(1) var shadowmap_sampler: sampler_comparison;



fn sample_shadows(world_pos: vec3<f32>) -> f32 {
	var shadowmap_pos = shadowmap_proj_mat * vec4<f32>(world_pos, 1.0);
	shadowmap_pos = vec4<f32>(shadowmap_pos.xy * vec2<f32>(0.5, -0.5) + 0.5, shadowmap_pos.z, 1.0);
	return textureSampleCompareLevel(shadowmap_texture, shadowmap_sampler, shadowmap_pos.xy, shadowmap_pos.z);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
	let color = textureSample(material_texture, material_sampler, in.texcoords);
	var color_rgb = color.rgb;
	let color_a = color.a;
	
	let ambient_light = vec3<f32>(0.9, 0.9, 1.0) * 0.5;
	let shadowcaster_light = vec3<f32>(1.0, 1.0, 0.9) * sample_shadows(in.world_pos);
	color_rgb *= ambient_light + shadowcaster_light;
	
	return vec4<f32>(color_rgb, color.a);
}

@group(0) @binding(0) var<uniform> camera_data: CameraData;
@group(0) @binding(1) var<uniform> shadow_caster_proj_mat: mat4x4f;
@group(0) @binding(2) var material_sampler: sampler;
@group(0) @binding(3) var shadowmap_texture: texture_depth_2d;
@group(0) @binding(4) var shadowmap_sampler: sampler_comparison;

struct CameraData {
	proj_view_mat: mat4x4f,
	inv_proj_mat: mat4x4f,
	view_mat: mat4x4f,
}

struct BasicVertexInput {
	@location(0) position: vec3f,
}

struct ExtendedVertexInput {
	@location(1) texcoords: vec2f,
	@location(2) normal: vec2f,
}

struct InstanceInput {
	@location(3) model_mat_0: vec4f,
	@location(4) model_mat_1: vec4f,
	@location(5) model_mat_2: vec4f,
	@location(6) model_mat_3: vec4f,
};



@vertex
fn vs_main(
	vertex_basic: BasicVertexInput,
	vertex_extended: ExtendedVertexInput,
	instance: InstanceInput,
) -> VertexOutput {
	
	let instance_mat = mat4x4(
		instance.model_mat_0,
		instance.model_mat_1,
		instance.model_mat_2,
		instance.model_mat_3,
	);
	
	var world_pos = instance_mat * vec4(vertex_basic.position, 1.0);
	
	var out: VertexOutput;
	out.screen_pos = camera_data.proj_view_mat * world_pos;
	out.screen_pos.z = out.screen_pos.z * 0.5 + 0.5;
	out.world_pos = world_pos.xyz;
	out.texcoords = vertex_extended.texcoords;
	return out;
}



struct VertexOutput {
	@builtin(position) screen_pos: vec4f,
	@location(0) world_pos: vec3f,
	@location(1) texcoords: vec2f,
};

@group(1) @binding(0) var material_texture: texture_2d<f32>;



fn sample_shadows(world_pos: vec3f) -> f32 {
	var shadowmap_pos = shadow_caster_proj_mat * vec4(world_pos, 1.0);
	// shadowmap_pos starts in range -1 to 1 with y going up, but we need 0 to 1 with y going down
	shadowmap_pos = vec4(shadowmap_pos.xyz * vec3(0.5, -0.5, 0.5) + 0.5, 1.0);
	return textureSampleCompareLevel(shadowmap_texture, shadowmap_sampler, shadowmap_pos.xy, shadowmap_pos.z);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
	let color = textureSample(material_texture, material_sampler, in.texcoords);
	var color_rgb = color.rgb;
	let color_a = color.a;
	
	let ambient_light = vec3(0.9, 0.9, 1.0) * 0.5;
	let shadowcaster_light = vec3(1.0, 0.9, 0.7) * sample_shadows(in.world_pos);
	color_rgb *= ambient_light + shadowcaster_light;
	
	return vec4(color_rgb, color.a);
}

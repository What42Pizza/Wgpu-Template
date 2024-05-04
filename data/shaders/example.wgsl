@group(0) @binding(0) var<uniform> camera_data: CameraData;

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
	
	let model_mat = mat4x4<f32>(
		instance.model_mat_0,
		instance.model_mat_1,
		instance.model_mat_2,
		instance.model_mat_3,
	);
	
	var out: VertexOutput;
	out.texcoords = vertex_extended.texcoords;
	out.pos = camera_data.proj_view_mat * model_mat * vec4<f32>(vertex_basic.position, 1.0);
	out.pos.z = out.pos.z * 0.5 + 0.25;
	return out;
}



struct VertexOutput {
	@builtin(position) pos: vec4<f32>,
	@location(0) texcoords: vec2<f32>,
};

@group(1) @binding(0) var material_texture: texture_2d<f32>;
@group(1) @binding(1) var material_sampler: sampler;



@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
	return textureSample(material_texture, material_sampler, in.texcoords);
}

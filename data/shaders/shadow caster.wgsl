@group(0) @binding(0) var<uniform> proj_mat: mat4x4f;

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
) -> @builtin(position) vec4f {
	
	let instance_mat = mat4x4(
		instance.model_mat_0,
		instance.model_mat_1,
		instance.model_mat_2,
		instance.model_mat_3,
	);
	
	var out = proj_mat * instance_mat * vec4(vertex_basic.position, 1.0);
	out.z = out.z * 0.5 + 0.5;
	return out;
}

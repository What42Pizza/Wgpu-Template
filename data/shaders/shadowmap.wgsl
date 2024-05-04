@group(0) @binding(0) var<uniform> proj_mat: mat4x4<f32>;

struct InstanceInput {
	@location(3) model_mat_0: vec4<f32>,
	@location(4) model_mat_1: vec4<f32>,
	@location(5) model_mat_2: vec4<f32>,
	@location(6) model_mat_3: vec4<f32>,
};



@vertex
fn vs_main(
	@location(0) position: vec3<f32>,
	instance: InstanceInput,
) -> @builtin(position) vec4<f32> {
	
	let model_mat = mat4x4<f32>(
		instance.model_mat_0,
		instance.model_mat_1,
		instance.model_mat_2,
		instance.model_mat_3,
	);
	
	var out = proj_mat * model_mat * vec4<f32>(position, 1.0);
	out.z = out.z * 0.5 + 0.25;
	return out;
}



//struct VertexOutput {
//	@builtin(position) pos: vec4<f32>,
//};



//@fragment
//fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
//	return textureSample(material_texture, material_sampler, in.texcoords);
//}

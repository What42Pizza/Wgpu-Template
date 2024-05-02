@group(0) @binding(0) var<uniform> camera_data: CameraData;

struct CameraData {
	proj_view_mat: mat4x4<f32>,
	inv_proj_mat: mat4x4<f32>,
	view_mat: mat4x4<f32>,
}



@vertex
fn vs_main(
	@builtin(vertex_index) index: u32
) -> VertexOutput {
	
	// hacky way to draw a single large triangle that convers the entire screen
	let pos = vec4<f32>(
		f32(i32(index) / 2) * 4.0 - 1.0,
		f32(i32(index) & 1) * 4.0 - 1.0,
		1.0,
		1.0
	);
	let inv_view_mat = transpose(mat3x3<f32>(camera_data.view_mat[0].xyz, camera_data.view_mat[1].xyz, camera_data.view_mat[2].xyz));
	
	let camera_pos = camera_data.inv_proj_mat * pos;
	let world_pos = inv_view_mat * camera_pos.xyz;
	
	var out: VertexOutput;
	out.texcoords = world_pos;
	out.pos = pos;
	out.pos.z = out.pos.z * 0.5 + 0.25;
	return out;
}



struct VertexOutput {
	@builtin(position) pos: vec4<f32>,
	@location(0) texcoords: vec3<f32>,
};

@group(1) @binding(0) var skybox_texture: texture_cube<f32>;
@group(1) @binding(1) var skybox_sampler: sampler;



@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
	return textureSample(skybox_texture, skybox_sampler, in.texcoords);
	//return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}

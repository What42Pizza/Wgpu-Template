@group(0) @binding(0) var<uniform> camera_data: CameraData;

struct CameraData {
	proj_view_mat: mat4x4f,
	inv_proj_mat: mat4x4f,
	view_mat: mat4x4f,
}



@vertex
fn vs_main(
	@builtin(vertex_index) index: u32
) -> VertexOutput {
	
	// hacky way to draw a single large triangle that convers the entire screen
	let screen_pos = vec4(
		f32(i32(index) / 2) * 4.0 - 1.0,
		f32(i32(index) & 1) * 4.0 - 1.0,
		1.0,
		1.0
	);
	let inv_view_mat = transpose(mat3x3(camera_data.view_mat[0].xyz, camera_data.view_mat[1].xyz, camera_data.view_mat[2].xyz));
	
	let camera_pos = camera_data.inv_proj_mat * screen_pos;
	let world_pos = inv_view_mat * camera_pos.xyz;
	
	var out: VertexOutput;
	out.screen_pos = screen_pos;
	out.screen_pos.z = out.screen_pos.z * 0.5 + 0.5;
	out.texcoords = world_pos;
	return out;
}



struct VertexOutput {
	@builtin(position) screen_pos: vec4f,
	@location(0) texcoords: vec3f,
};

@group(1) @binding(0) var skybox_texture: texture_cube<f32>;
@group(1) @binding(1) var skybox_sampler: sampler;



@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
	return textureSample(skybox_texture, skybox_sampler, in.texcoords);
}

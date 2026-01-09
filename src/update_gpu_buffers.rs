use crate::prelude::*;



pub fn update_gpu_buffers(program_data: &mut ProgramData) {
	
	let frustum_planes = get_frustum_planes(&program_data.camera_data, program_data.render_context.aspect_ratio);
	
	// culled_instances_buffer
	// NOTE: if you have multiple culls to do then it would probably benefit from
	// multithreading (with rayon::join, rayon::scope, etc), but from my own testing,
	// I get worse performance when this culling and the other buffer writes are done on
	// separate threads
	let instance_datas = &program_data.example_model_instance_datas;
	let bounding_radius = program_data.render_assets.example_models.bounding_radius;
	let mut new_model_instances_data = Vec::with_capacity(instance_datas.len());
	for (i, instance) in instance_datas.iter().enumerate() {
		if model_is_visible(&instance.pos, bounding_radius, &frustum_planes) {
			new_model_instances_data.push(program_data.example_model_instance_datas[i].to_raw());
		}
	}
	program_data.render_context.command_queue.write_buffer(
		&program_data.render_assets.example_models.culled_instances_buffer,
		0,
		bytemuck::cast_slice(&new_model_instances_data),
	);
	program_data.render_assets.example_models.culled_instances_count = new_model_instances_data.len() as u32;
	
	// camera.buffer
	let camera_gpu_data = program_data.camera_data.build_gpu_data(program_data.render_context.aspect_ratio);
	program_data.render_context.command_queue.write_buffer(
		&program_data.render_assets.camera.buffer,
		0,
		bytemuck::cast_slice(&camera_gpu_data),
	);
	
	// shadow_caster.proj_mat_buffer
	let shadow_caster_gpu_data = program_data.shadow_caster_data.build_gpu_data(program_data.camera_data.pos);
	program_data.render_context.command_queue.write_buffer(
		&program_data.render_assets.shadow_caster.proj_mat_buffer,
		0,
		bytemuck::cast_slice(&shadow_caster_gpu_data),
	);
	
}



pub fn get_frustum_planes(camera: &CameraData, aspect_ratio: f32) -> [(glam::Vec3, f32); 5] {
	let forward = glam::Vec3::new(
		camera.rot_xz.cos() * camera.rot_y.cos(),
		camera.rot_y.sin(),
		camera.rot_xz.sin() * camera.rot_y.cos(),
	).normalize();
	
	let up_dir = glam::Vec3::Y;
	let right_dir = forward.cross(up_dir).normalize();
	let up_dir = forward.cross(-right_dir).normalize();
	const FOV_MULT: f32 = 1.0; // lower this to see the culling work
	let half_height = (camera.fov_radians * FOV_MULT / 2.0).tan() * camera.near;
	let half_width = half_height * aspect_ratio;
	let near_plane_center = camera.pos + forward * camera.near;
	
	let left_plane = {
		let point_on_plane = near_plane_center - right_dir * half_width;
		let normal = -up_dir.cross(point_on_plane - camera.pos).normalize();
		let dist = point_on_plane.dot(normal);
		(normal, dist)
	};
	
	let right_plane = {
		let point_on_plane = near_plane_center + right_dir * half_width;
		let normal = up_dir.cross(point_on_plane - camera.pos).normalize();
		let dist = point_on_plane.dot(normal);
		(normal, dist)
	};
	
	let top_plane = {
		let point_on_plane = near_plane_center + up_dir * half_height;
		let normal = -right_dir.cross(point_on_plane - camera.pos).normalize();
		let dist = point_on_plane.dot(normal);
		(normal, dist)
	};
	
	let bottom_plane = {
		let point_on_plane = near_plane_center - up_dir * half_height;
		let normal = right_dir.cross(point_on_plane - camera.pos).normalize();
		let dist = point_on_plane.dot(normal);
		(normal, dist)
	};
	
	//let near_plane = { // there's not really any point in checking this plane but you still can if you just want to
	//	let normal = forward;
	//	let point_on_plane = camera.pos + forward * camera.near;
	//	let dist = point_on_plane.dot(forward);
	//	(normal, dist)
	//};
	
	let far_plane = { // you might be able to remove this one too, I have it last b/c it should be the least likely to trigger removal
		let normal = -forward;
		let point_on_plane = camera.pos + forward * camera.far;
		let dist = point_on_plane.dot(normal);
		(normal, dist)
	};
	
	[far_plane, left_plane, right_plane, top_plane, bottom_plane]
}



pub fn model_is_visible(pos: &glam::Vec3, bounding_radius: f32, frustum_planes: &[(glam::Vec3, f32); 5]) -> bool {
	is_sphere_past_plane(pos, bounding_radius, &frustum_planes[0].0, frustum_planes[0].1)
		&& is_sphere_past_plane(pos, bounding_radius, &frustum_planes[1].0, frustum_planes[1].1)
		&& is_sphere_past_plane(pos, bounding_radius, &frustum_planes[2].0, frustum_planes[2].1)
		&& is_sphere_past_plane(pos, bounding_radius, &frustum_planes[3].0, frustum_planes[3].1)
		&& is_sphere_past_plane(pos, bounding_radius, &frustum_planes[4].0, frustum_planes[4].1)
}

pub fn is_sphere_past_plane(pos: &glam::Vec3, radius: f32, plane_normal: &glam::Vec3, plane_dist: f32) -> bool {
	plane_normal.dot(*pos) > plane_dist - radius
}

use crate::prelude::*;
use winit::keyboard::KeyCode;



pub fn update(program_data: &mut ProgramData, dt: f32) -> Result<()> {
	
	update_camera(program_data, dt);
	
	Ok(())
}



fn update_camera(program_data: &mut ProgramData, dt: f32) {
	let speed = 10.0 * dt;
	use cgmath::InnerSpace;
	let forward = program_data.camera.target - program_data.camera.eye;
	let forward_norm = forward.normalize();
	let forward_mag = forward.magnitude();
	
	if program_data.key_is_down(KeyCode::KeyW) && forward_mag > speed {
		program_data.camera.eye += forward_norm * speed;
	}
	if program_data.key_is_down(KeyCode::KeyS) {
		program_data.camera.eye -= forward_norm * speed;
	}
	
	let right = forward_norm.cross(program_data.camera.up);
	
	let forward = program_data.camera.target - program_data.camera.eye;
	let forward_mag = forward.magnitude();
	
	if program_data.key_is_down(KeyCode::KeyD) {
		program_data.camera.eye = program_data.camera.target - (forward + right * speed).normalize() * forward_mag;
	}
	if program_data.key_is_down(KeyCode::KeyA) {
		program_data.camera.eye = program_data.camera.target - (forward - right * speed).normalize() * forward_mag;
	}
	
	let new_camera_data: [[f32; 4]; 4] = program_data.camera.build_view_projection_matrix(program_data.render_context.aspect_ratio).into();
	program_data.render_context.command_queue.write_buffer(
		&program_data.uniform_datas.camera_binding.buffer,
		0,
		bytemuck::cast_slice(&[new_camera_data]),
	);
	
}

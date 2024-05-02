use crate::prelude::*;
use winit::keyboard::KeyCode;



pub fn update(program_data: &mut ProgramData, dt: f32) -> Result<()> {
	
	update_camera(program_data, dt);
	
	Ok(())
}



fn update_camera(program_data: &mut ProgramData, dt: f32) {
	let speed = 30.0 * dt;
	let forward = program_data.camera.target - program_data.camera.eye;
	let forward_dir = forward.normalize();
	let forward_mag = forward.length();
	
	if program_data.key_is_down(KeyCode::KeyW) && forward_mag > speed {
		program_data.camera.eye += forward_dir * speed;
	}
	if program_data.key_is_down(KeyCode::KeyS) {
		program_data.camera.eye -= forward_dir * speed;
	}
	
	let right_dir = forward_dir.cross(program_data.camera.up);
	
	let forward = program_data.camera.target - program_data.camera.eye;
	let forward_mag = forward.length();
	
	if program_data.key_is_down(KeyCode::KeyD) {
		program_data.camera.eye += right_dir * speed;
	}
	if program_data.key_is_down(KeyCode::KeyA) {
		program_data.camera.eye -= right_dir * speed;
	}
	
	if program_data.key_is_down(KeyCode::KeyE) {
		program_data.camera.eye.y += speed;
	}
	if program_data.key_is_down(KeyCode::KeyQ) {
		program_data.camera.eye.y -= speed;
	}
	
	let view_poj_mat = program_data.camera.build_data(program_data.render_context.aspect_ratio);
	program_data.render_context.command_queue.write_buffer(
		&program_data.render_assets.camera.buffer,
		0,
		bytemuck::cast_slice(&view_poj_mat),
	);
	
}

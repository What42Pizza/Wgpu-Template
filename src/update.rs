use crate::prelude::*;
use winit::keyboard::KeyCode;



pub type ShouldExit = bool;

pub fn update(program_data: &mut ProgramData, dt: f32) -> Result<ShouldExit> {
	
	if program_data.input.key_just_pressed(KeyCode::Escape) {
		return Ok(true);
	}
	
	if program_data.input.is_focused {
		update_camera(program_data, dt);
	}
	
	update_gpu_buffers(program_data);
	
	Ok(false)
}



fn update_camera(program_data: &mut ProgramData, dt: f32) {
	let input = &program_data.input;
	let camera_data = &mut program_data.camera_data;
	let mut speed = 30.0 * dt;
	if program_data.input.key_is_down(KeyCode::ShiftLeft) {
		speed *= 5.0;
	}
	let forward = glam::Vec3::new(
		camera_data.rot_xz.cos() * camera_data.rot_y.cos(),
		camera_data.rot_y.sin(),
		camera_data.rot_xz.sin() * camera_data.rot_y.cos(),
	);
	let forward_dir = forward.normalize();
	let right_dir = forward_dir.cross(glam::Vec3::Y).normalize();
	
	if input.key_is_down(KeyCode::KeyW) {
		camera_data.pos += forward_dir * speed;
	}
	if input.key_is_down(KeyCode::KeyS) {
		camera_data.pos -= forward_dir * speed;
	}
	
	if input.key_is_down(KeyCode::KeyD) {
		camera_data.pos += right_dir * speed;
	}
	if input.key_is_down(KeyCode::KeyA) {
		camera_data.pos -= right_dir * speed;
	}
	
	if input.key_is_down(KeyCode::KeyE) {
		camera_data.pos.y += speed;
	}
	if input.key_is_down(KeyCode::KeyQ) {
		camera_data.pos.y -= speed;
	}
	
	let sensitivity = 0.005;
	let mouse_dt = (
		(input.mouse_pos.x - input.prev_mouse_pos.x).clamp(-50.0, 50.0) as f32 * sensitivity,
		(input.mouse_pos.y - input.prev_mouse_pos.y).clamp(-50.0, 50.0) as f32 * sensitivity,
	);
	camera_data.rot_xz += mouse_dt.0;
	camera_data.rot_y = (camera_data.rot_y - mouse_dt.1).clamp(-std::f32::consts::FRAC_PI_2 * 0.999, std::f32::consts::FRAC_PI_2 * 0.999);
	
	
}



pub fn update_gpu_buffers(program_data: &mut ProgramData) {
	
	let camera_gpu_data = program_data.camera_data.build_gpu_data(program_data.render_context.aspect_ratio);
	program_data.render_context.command_queue.write_buffer(
		&program_data.render_assets.camera.buffer,
		0,
		bytemuck::cast_slice(&camera_gpu_data),
	);
	
	let shadow_caster_gpu_data = program_data.shadow_caster_data.build_gpu_data(program_data.camera_data.pos);
	program_data.render_context.command_queue.write_buffer(
		&program_data.render_assets.shadow_caster.proj_mat_buffer,
		0,
		bytemuck::cast_slice(&shadow_caster_gpu_data),
	);
	
}

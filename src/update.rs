use crate::prelude::*;
use winit::keyboard::KeyCode;



pub type ShouldExit = bool;

pub fn update(program_data: &mut ProgramData, dt: f32) -> Result<ShouldExit> {
	
	if program_data.key_is_down(KeyCode::Escape) {
		return Ok(true);
	}
	
	update_camera(program_data, dt);
	
	update_gpu_buffers(program_data);
	
	Ok(false)
}



fn update_camera(program_data: &mut ProgramData, dt: f32) {
	let speed = 30.0 * dt;
	let forward = program_data.camera_data.target - program_data.camera_data.eye;
	let forward_dir = forward.normalize();
	let right_dir = forward_dir.cross(program_data.camera_data.up);
	
	if program_data.key_is_down(KeyCode::KeyW) && speed < forward.length() {
		program_data.camera_data.eye += forward_dir * speed;
	}
	if program_data.key_is_down(KeyCode::KeyS) {
		program_data.camera_data.eye -= forward_dir * speed;
	}
	
	if program_data.key_is_down(KeyCode::KeyD) {
		program_data.camera_data.eye += right_dir * speed;
	}
	if program_data.key_is_down(KeyCode::KeyA) {
		program_data.camera_data.eye -= right_dir * speed;
	}
	
	if program_data.key_is_down(KeyCode::KeyE) {
		program_data.camera_data.eye.y += speed;
	}
	if program_data.key_is_down(KeyCode::KeyQ) {
		program_data.camera_data.eye.y -= speed;
	}
	
}



pub fn update_gpu_buffers(program_data: &mut ProgramData) {
	
	let camera_gpu_data = program_data.camera_data.build_gpu_data(program_data.render_context.aspect_ratio);
	program_data.render_context.command_queue.write_buffer(
		&program_data.render_assets.camera.buffer,
		0,
		bytemuck::cast_slice(&camera_gpu_data),
	);
	
	// this code isn't actually used right now, but it adds the ability to update the shadow projection matrix by setting `shader_caster_data.is_dirty`
	let shadow_caster = &mut program_data.render_assets.shadow_caster;
	if shadow_caster.is_dirty {
		shadow_caster.is_dirty = false;
		let shadow_caster_gpu_data = program_data.shadow_caster_data.build_gpu_data();
		program_data.render_context.command_queue.write_buffer(
			&program_data.render_assets.camera.buffer,
			0,
			bytemuck::cast_slice(&shadow_caster_gpu_data),
		);
	}
	
}

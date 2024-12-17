use crate::prelude::*;
use winit::{dpi::PhysicalPosition, keyboard::KeyCode};



pub type ShouldExit = bool;

pub fn update(program_data: &mut ProgramData, dt: f32) -> Result<ShouldExit> {
	let is_focused = program_data.render_context.window.has_focus();
	
	let should_exit = process_pre_frame_inputs(program_data);
	if should_exit {return Ok(true);}
	
	if is_focused && program_data.is_moving_camera {
		update_camera(program_data, dt);
	}
	//program_data.camera_data.rot_xz = 0.5; // for benchmarking
	//program_data.camera_data.rot_y = 0.0;
	
	let should_exit = process_post_frame_inputs(program_data);
	if should_exit {return Ok(true);}
	
	Ok(false)
}



pub fn process_pre_frame_inputs(program_data: &mut ProgramData) -> ShouldExit {
	let window = program_data.render_context.window;
	let input = &program_data.input;
	let shift_down = input.key_is_down(KeyCode::ShiftLeft) || input.key_is_down(KeyCode::ShiftRight);
	let control_down = input.key_is_down(KeyCode::ControlLeft) || input.key_is_down(KeyCode::ControlRight);
	let alt_down = input.key_is_down(KeyCode::AltLeft) || input.key_is_down(KeyCode::AltRight);
	
	// ctrl+w exit
	if control_down && input.key_just_pressed(KeyCode::KeyW) {
		return true;
	}
	
	// esc to lose camera focus
	if input.key_just_pressed(KeyCode::Escape) {
		window.set_cursor_visible(true);
		program_data.is_moving_camera = false;
		program_data.input.capture_cursor = false;
	}
	
	false
}



pub fn process_post_frame_inputs(program_data: &mut ProgramData) -> ShouldExit {
	let window = program_data.render_context.window;
	let input = &program_data.input;
	
	// click to gain camera focus
	if input.button_just_pressed(MouseButton::Left) {
		window.set_cursor_visible(false);
		program_data.is_moving_camera = true;
		program_data.input.capture_cursor = true;
		let size = program_data.render_context.surface_size;
		let window_center = PhysicalPosition::new(size.width as f64 / 2.0, size.height as f64 / 2.0);
		let _ = program_data.render_context.window.set_cursor_position(window_center);
	}
	
	false
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
		input.mouse_vel.x.clamp(-50.0, 50.0) as f32 * sensitivity,
		input.mouse_vel.y.clamp(-50.0, 50.0) as f32 * sensitivity,
	);
	camera_data.rot_xz += mouse_dt.0;
	camera_data.rot_y = (camera_data.rot_y - mouse_dt.1).clamp(-std::f32::consts::FRAC_PI_2 * 0.999, std::f32::consts::FRAC_PI_2 * 0.999);
	
}

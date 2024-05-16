// Started:      24/04/18
// Last updated: 24/05/12

// Learn Wgpu website: https://sotrh.github.io/learn-wgpu/
// Learn Wgpu repo: https://github.com/sotrh/learn-wgpu
// Skybox texture: https://opengameart.org/content/clouds-skybox-1



#![feature(duration_constants)]
#![feature(let_chains)]

#![allow(unused)]
#![warn(unused_must_use)]

#![allow(unused_doc_comments)]
#![allow(clippy::new_without_default)]
#![warn(clippy::todo)]
#![deny(clippy::unwrap_used, clippy::panic)]

// enable to see uses
//#[warn(clippy::expect_used)]



pub mod load;
pub mod update;
pub mod render;
pub mod data;
pub mod materials_storage_utils;
pub mod utils;

pub mod prelude {
	pub use crate::{*, data::*, utils::IoResultFns};
	pub use std::{
		fs,
		collections::{HashMap, HashSet},
		path::{Path, PathBuf},
		time::{Duration, Instant}
	};
	pub use std::result::Result as StdResult;
	pub use log::{info, warn, debug, error};
	pub use anyhow::*;
}

use crate::prelude::*;
use std::{env, thread};
use winit::{
	application::ApplicationHandler,
	dpi::PhysicalSize,
	event::{ElementState, KeyEvent, MouseButton, WindowEvent},
	event_loop::{ActiveEventLoop, EventLoop},
	keyboard::PhysicalKey,
	platform::pump_events::EventLoopExtPumpEvents,
	window::{Window, WindowId}
};



fn main() -> Result<()> {
	let start_time = Instant::now();
	
	if env::var("RUST_LOG").is_err() {
		env::set_var("RUST_LOG", "warn");
	}
	env_logger::init();
	
	/// With Winit 0.30.0, there's kinda a catch-22 here where A: we need the window to be
	/// available before we create the application struct, B: we need the application
	/// struct in order to start the event loop, and C: we need to start the event loop to
	/// create a window. So, we use EventLoopExtPumpEvents::pump_app_events to run the
	/// event loop until we can get a window, then use that to create the application
	/// struct, then use that to start the event loop. Although, I've heard that you can
	/// also store the window in an Option<Arc<>>, which allows you to store both the
	/// window and render context is the main state struct
	info!("Running initialization event_loop...");
	let mut event_loop = EventLoop::new().context("Failed to create event loop.")?;
	let mut init_data = InitData::default();
	let window = loop {
		event_loop.pump_app_events(None, &mut init_data);
		if let Some(window) = init_data.window {
			break window;
		}
	};
	
	window.focus_window();
	window.set_cursor_visible(false);
	
	info!("Done, starting main event_loop...");
	let mut program_data = load::load_program_data(start_time, &window)?;
	event_loop.run_app(&mut program_data)?;
	
	Ok(())
}





/// the entire purpose of this part is to get a usable window

#[derive(Default)]
pub struct InitData {
	pub window: Option<Window>,
}

impl ApplicationHandler for InitData {
	
	fn resumed(&mut self, event_loop: &ActiveEventLoop) {
		if self.window.is_none() {
			let window_attributes = Window::default_attributes()
				.with_title("WGPU Testing")
				.with_inner_size(PhysicalSize::new(1280, 720));
			let window = event_loop.create_window(window_attributes).expect("Could not init window.");
			window.request_redraw();
			self.window = Some(window);
		}
	}
	
	fn window_event(
		&mut self,
		_event_loop: &ActiveEventLoop,
		_window_id: WindowId,
		_event: WindowEvent,
	) {}
	
	fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {}
}










impl<'a> ApplicationHandler for ProgramData<'a> {
	
	
	
	fn resumed(&mut self, _event_loop: &ActiveEventLoop) {
		warn!("Resumed, idk what to do here");
	}
	
	
	
	fn window_event(
		&mut self,
		event_loop: &ActiveEventLoop,
		_window_id: WindowId,
		event: WindowEvent,
	) {
		let program_data = self;
		
		match event {
			
			WindowEvent::RedrawRequested => {
				let result = redraw_requested(program_data, event_loop);
				if let Err(err) = result {
					error!("Fatal error while processing frame: {err}");
					event_loop.exit();
				}
			}
			
			WindowEvent::Resized (new_size) => {
				resize(program_data, new_size).expect("Failed to resize the window");
			}
			
			WindowEvent::CloseRequested => {
				event_loop.exit();
			}
			
			WindowEvent::Focused (is_focused) => {
				program_data.input.window_is_focused = is_focused;
			}
			
			WindowEvent::KeyboardInput {
				event: KeyEvent {
					physical_key: PhysicalKey::Code (key),
					state,
					..
				},
				..
			} => {
				if state.is_pressed() {
					program_data.input.pressed_keys.insert(key);
				} else {
					program_data.input.pressed_keys.remove(&key);
				}
			}
			
			WindowEvent::CursorMoved {device_id: _, position} => {
				program_data.input.mouse_pos = position;
			}
			
			WindowEvent::MouseInput {device_id: _, state, button} => {
				let mouse_buttons = &mut program_data.input.pressed_mouse_buttons;
				match button {
					MouseButton::Left    => mouse_buttons.left_is_down    = state.is_pressed(),
					MouseButton::Right   => mouse_buttons.right_is_down   = state.is_pressed(),
					MouseButton::Middle  => mouse_buttons.middle_is_down  = state.is_pressed(),
					MouseButton::Back    => mouse_buttons.back_is_down    = state.is_pressed(),
					MouseButton::Forward => mouse_buttons.forward_is_down = state.is_pressed(),
					MouseButton::Other (id) => {
						if state.is_pressed() {
							mouse_buttons.others_down.insert(id);
						} else {
							mouse_buttons.others_down.remove(&id);
						}
					},
				}
			}
			
			_ => (),
		}
	}
	
	
	
	fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
		let program_data = self;
		program_data.render_context.window.request_redraw();
	}
	
}



pub fn resize(program_data: &mut ProgramData, new_size: PhysicalSize<u32>) -> Result<()> {
	let render_context = &mut program_data.render_context;
	render_context.surface_size = new_size;
	render_context.aspect_ratio = new_size.width as f32 / new_size.height as f32;
	render_context.surface_config.width = new_size.width;
	render_context.surface_config.height = new_size.height;
	if new_size.width == 0 || new_size.height == 0 {return Ok(());}
	render_context.drawable_surface.configure(&render_context.device, &render_context.surface_config);
	program_data.render_assets.depth = load::load_depth_render_data(render_context);
	Ok(())
}





pub fn redraw_requested(program_data: &mut ProgramData, event_loop: &ActiveEventLoop) -> Result<()> {
	
	
	let frame_start_time = Instant::now();
	
	let dt = program_data.step_dt();
	let should_exit = update::update(program_data, dt)?;
	if should_exit {
		event_loop.exit();
		return Ok(());
	}
	
	
	// make sure to only render when the window is visible
	let render_context = &program_data.render_context;
	let size = render_context.surface_size;
	if size.width > 0 && size.height > 0 {
		
		
		let surface_output_result = render_context.drawable_surface.get_current_texture();
		let surface_output = match surface_output_result {
			StdResult::Ok(v) => v,
			StdResult::Err(wgpu::SurfaceError::Lost) => {
				warn!("Surface was lost, attempting to resize...");
				resize(program_data, render_context.surface_size).context("Failed to resize window.")?;
				program_data.render_context.drawable_surface.get_current_texture().context("Failed to get current window drawable texture, even after resize.")?
			}
			StdResult::Err(wgpu::SurfaceError::Outdated) => {
				warn!("Surface is outdated, attempting to resize...");
				resize(program_data, render_context.surface_size).context("Failed to resize window.")?;
				program_data.render_context.drawable_surface.get_current_texture().context("Failed to get current window drawable texture, even after resize.")?
			}
			StdResult::Err(wgpu::SurfaceError::OutOfMemory) => {
				warn!("OutOfMemory error while rendering, exiting process.");
				event_loop.exit();
				return Ok(());
			}
			StdResult::Err(err) => return Err(err.into()),
		};
		
		render::render(&surface_output, program_data);
		
		
		let frame_time = frame_start_time.elapsed();
		let min_frame_time = program_data.engine_config.min_frame_time;
		if frame_time < min_frame_time {
			let sleep_time = min_frame_time - frame_time;
			thread::sleep(sleep_time);
		}
		
		let fps_counter_output = program_data.fps_counter.step(frame_start_time.elapsed());
		if let Some((average_fps, average_frame_time)) = fps_counter_output {
			println!("FPS: {average_fps}  (avg frame time: {average_frame_time:?})");
		}
		
		
		program_data.render_context.window.pre_present_notify();
		surface_output.present();
		
		let input = &mut program_data.input;
		input.prev_mouse_pos = input.mouse_pos;
		input.prev_pressed_keys.clone_from(&input.pressed_keys);
		input.prev_pressed_mouse_buttons = input.pressed_mouse_buttons.clone();
		
		
	}
	
	
	Ok(())
}

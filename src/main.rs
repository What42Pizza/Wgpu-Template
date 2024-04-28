// started:      24/04/18
// last updated: 24/04/28



#![feature(duration_constants)]
#![feature(let_chains)]

#![allow(unused)]
#![warn(unused_must_use)]

#![allow(clippy::new_without_default)]
#![warn(clippy::todo)]
#![deny(clippy::unwrap_used, clippy::panic)]

// enable to see uses
//#[warn(clippy::expect_used)]



pub mod load;
pub mod render;
pub mod wgpu_integration;
pub mod data;
pub mod utils;

pub mod prelude {
	pub use crate::{*, data::*};
	pub use std::{fs, path::{Path, PathBuf}, time::{Duration, Instant}};
	pub use std::result::Result as StdResult;
	//pub use log::{info, warn};
	pub use anyhow::*;
}

use crate::prelude::*;
use std::mem;
use winit::{
	application::ApplicationHandler,
	dpi::PhysicalSize,
	platform::pump_events::EventLoopExtPumpEvents,
	event::{ElementState, KeyEvent, WindowEvent},
	event_loop::{ActiveEventLoop, EventLoop},
	window::{Window, WindowId},
};



fn main() -> Result<()> {
	let start_time = Instant::now();
	env_logger::init();
	
	let mut event_loop = EventLoop::new()?;
	let mut init_data = InitData::default();
	
	log!("Running initialization event_loop...");
	let window = loop {
		event_loop.pump_app_events(None, &mut init_data);
		if let Some(window) = mem::take(&mut init_data.window) {
			break window;
		}
	};
	
	log!("Done, starting main event_loop...");
	let mut program_data = load::init_program_data(start_time, &window)?;
	event_loop.run_app(&mut program_data)?;
	
	log!(>flush);
	Ok(())
}





#[derive(Default)]
pub struct InitData {
	pub window: Option<Window>,
}

impl ApplicationHandler for InitData {
	//fn new_events(&mut self, _event_loop: &ActiveEventLoop, cause: StartCause) {
	//	log!("new_events: {cause:?}");
	//}
	
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
	//fn new_events(&mut self, _event_loop: &ActiveEventLoop, cause: StartCause) {
	//	log!("new_events: {cause:?}");
	//}
	
	
	
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
			
			WindowEvent::CloseRequested => {
				event_loop.exit();
			},
			
			WindowEvent::KeyboardInput {
				event: KeyEvent { logical_key: key, state: ElementState::Pressed, .. },
				..
			} => match key.as_ref() {
				// WARNING: Consider using `key_without_modifiers()` if available on your platform.
				// See the `key_binding` example
				_ => (),
			},
			
			WindowEvent::RedrawRequested => {
				
				let frame_start_time = Instant::now();
				let render_result = render::render(program_data);
				match render_result {
					StdResult::Ok(_) => {}
					StdResult::Err(wgpu::SurfaceError::Lost) => {
						let size = program_data.render_context.size;
						resize(&mut program_data.render_context, size).expect("Could not resize window.");
					}
					StdResult::Err(wgpu::SurfaceError::OutOfMemory) => {
						warn!("OutOfMemory error while rendering, exiting process.");
						event_loop.exit();
					},
					StdResult::Err(e) => warn!("Error while rendering: {e:?}"),
				}
				let fps_counter_output = program_data.fps_counter.step(frame_start_time.elapsed());
				if let Some((average_fps, average_frame_time)) = fps_counter_output {
					log!("FPS: {average_fps}  (frame time: {average_frame_time:?})");
				}
				
			},
			
			WindowEvent::Resized (new_size) => {
				resize(&mut program_data.render_context, new_size).expect("Failed to resize the window");
			}
			
			_ => (),
		}
	}
	
	
	
	fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
		let program_data = self;
		program_data.window.request_redraw();
	}
}



pub fn resize(render_context: &mut wgpu_integration::RenderContextData, new_size: PhysicalSize<u32>) -> Result<()> {
	if new_size.width == 0 {return Err(Error::msg("Width cannot be 0"));}
	if new_size.height == 0 {return Err(Error::msg("Height cannot be 0"));}
	render_context.size = new_size;
	render_context.aspect_ratio = new_size.width as f32 / new_size.height as f32;
	render_context.surface_config.width = new_size.width;
	render_context.surface_config.height = new_size.height;
	render_context.drawable_surface.configure(&render_context.device, &render_context.surface_config);
	Ok(())
}

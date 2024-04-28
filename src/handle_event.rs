use crate::prelude::*;
use winit::{event::{Event, WindowEvent}, event_loop::EventLoopWindowTarget, window::Window};



pub fn handle_event(event: &mut Event<()>, elwt: &EventLoopWindowTarget<()>, program_data: &mut ProgramData, window: &Window) -> Result<()> {
	match event {
		
		Event::AboutToWait => {
			window.request_redraw();
		}
		
		Event::WindowEvent {event: window_event, window_id, ..} if *window_id == window.id() => match window_event {
			
			WindowEvent::RedrawRequested => {
				let frame_start_time = Instant::now();
				let render_result = render::render(program_data);
				match render_result {
					StdResult::Ok(_) => {}
					StdResult::Err(wgpu::SurfaceError::Lost) => {
						let size = program_data.render_context.size;
						resize(&mut program_data.render_context, size)?;
					}
					StdResult::Err(wgpu::SurfaceError::OutOfMemory) => {
						log!("OutOfMemory error while rendering, exiting process.");
						elwt.exit();
					},
					StdResult::Err(e) => log!("Error while rendering: {e:?}"),
				}
				let fps_counter_output = program_data.fps_counter.step(frame_start_time.elapsed());
				if let Some((average_fps, average_frame_time)) = fps_counter_output {
					println!("FPS: {average_fps}  (frame time: {average_frame_time:?})");
				}
				//window.request_redraw();
			}
			
			WindowEvent::Resized (physical_size) => {
				let _ = resize(&mut program_data.render_context, *physical_size);
			}
			//WindowEvent::ScaleFactorChanged {inner_size_writer, ..} => {
			//	inner_size_writer.request_inner_size(program_data.wgpu_context.size)?;
			//}
			
			WindowEvent::CloseRequested => elwt.exit(),
			
			_ => {}
		},
		
		_ => {}
	}
	Ok(())
}



pub fn resize(wgpu: &mut wgpu_integration::RenderContextData, new_size: winit::dpi::PhysicalSize<u32>) -> Result<()> {
	if new_size.width == 0 {return Err(Error::msg("Width cannot be 0"));}
	if new_size.height == 0 {return Err(Error::msg("Height cannot be 0"));}
	wgpu.size = new_size;
	wgpu.surface_config.width = new_size.width;
	wgpu.surface_config.height = new_size.height;
	wgpu.drawable_surface.configure(&wgpu.device, &wgpu.surface_config);
	Ok(())
}

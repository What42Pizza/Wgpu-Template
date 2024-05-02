use crate::prelude::*;



pub fn render(output: &wgpu::SurfaceTexture, program_data: &mut ProgramData) -> StdResult<(), wgpu::SurfaceError> {
	let output_view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
	let encoder_descriptor = wgpu::CommandEncoderDescriptor {label: Some("Render Encoder")};
	let mut encoder = program_data.render_context.device.create_command_encoder(&encoder_descriptor);
	
	render_example_pipeline(program_data, &mut encoder, &output_view);
	render_skybox_pipeline(program_data, &mut encoder, &output_view); // it's better to have this at the end so that only the necessary pixels are rendered
	
	program_data.render_context.command_queue.submit(std::iter::once(encoder.finish()));
	
	StdResult::Ok(())
}



pub fn render_example_pipeline(program_data: &ProgramData, encoder: &mut wgpu::CommandEncoder, output_view: &wgpu::TextureView) {
	
	let mut example_pass_handle = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
		label: Some("Example Render Pass"),
		color_attachments: &[Some(wgpu::RenderPassColorAttachment {
			view: output_view,
			resolve_target: None,
			ops: wgpu::Operations {
				load: wgpu::LoadOp::Clear(wgpu::Color {
					r: 0.1,
					g: 0.2,
					b: 0.3,
					a: 1.0,
				}),
				store: wgpu::StoreOp::Store,
			},
		})],
		depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
			view: &program_data.render_assets.depth.view,
			depth_ops: Some(wgpu::Operations {
				load: wgpu::LoadOp::Clear(1.0),
				store: wgpu::StoreOp::Store,
			}),
			stencil_ops: None,
		}),
		occlusion_query_set: None,
		timestamp_writes: None,
	});
	
	example_pass_handle.set_pipeline(&program_data.render_pipelines.example);
	example_pass_handle.set_bind_group(0, &program_data.render_assets.camera.bind_group, &[]);
	example_pass_handle.set_bind_group(1, &program_data.render_assets.materials_storage.list_2d[program_data.render_assets.example_model.meshes[0].material_index].bind_group, &[]);
	
	let mesh = &program_data.render_assets.example_model.meshes[0];
	example_pass_handle.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
	example_pass_handle.set_vertex_buffer(1, program_data.render_assets.example_model.instances_buffer.slice(..));
	example_pass_handle.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
	example_pass_handle.draw_indexed(0..mesh.index_count, 0, 0..program_data.render_assets.example_model.instances_count);
	
}



pub fn render_skybox_pipeline(program_data: &ProgramData, encoder: &mut wgpu::CommandEncoder, output_view: &wgpu::TextureView) {
	
	let mut skybox_pass_handle = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
		label: Some("Skybox Render Pass"),
		color_attachments: &[Some(wgpu::RenderPassColorAttachment {
			view: output_view,
			resolve_target: None,
			ops: wgpu::Operations {
				load: wgpu::LoadOp::Load,
				store: wgpu::StoreOp::Store,
			},
		})],
		depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
			view: &program_data.render_assets.depth.view,
			depth_ops: Some(wgpu::Operations {
				load: wgpu::LoadOp::Load,
				store: wgpu::StoreOp::Store,
			}),
			stencil_ops: None,
		}),
		occlusion_query_set: None,
		timestamp_writes: None,
	});
	
	skybox_pass_handle.set_pipeline(&program_data.render_pipelines.skybox);
	skybox_pass_handle.set_bind_group(0, &program_data.render_assets.camera.bind_group, &[]);
	skybox_pass_handle.set_bind_group(1, &program_data.render_assets.materials_storage.list_cube[program_data.render_assets.skybox_material_index].bind_group, &[]);
	
	skybox_pass_handle.draw(0..3, 0..1)
	
}

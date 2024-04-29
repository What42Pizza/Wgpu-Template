use crate::prelude::*;



pub fn render(program_data: &mut ProgramData) -> StdResult<(), wgpu::SurfaceError> {
	let output = program_data.render_context.drawable_surface.get_current_texture()?;
	let output_view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
	let encoder_descriptor = wgpu::CommandEncoderDescriptor {label: Some("Render Encoder")};
	let mut encoder = program_data.render_context.device.create_command_encoder(&encoder_descriptor);
	
	let mut main_pass_handle = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
		label: Some("Main Render Pass"),
		color_attachments: &[Some(wgpu::RenderPassColorAttachment {
			view: &output_view,
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
		depth_stencil_attachment: None,
		occlusion_query_set: None,
		timestamp_writes: None,
	});
	main_pass_handle.set_pipeline(&program_data.render_pipelines.main);
	main_pass_handle.set_bind_group(0, &program_data.uniform_datas.camera_binding.group, &[]);
	main_pass_handle.set_bind_group(1, &program_data.asset_datas.happy_tree_binding.group, &[]);
	main_pass_handle.set_vertex_buffer(0, program_data.world_datas.main_vertices.slice(..));
	main_pass_handle.set_index_buffer(program_data.world_datas.main_indices.slice(..), wgpu::IndexFormat::Uint16);
	main_pass_handle.draw_indexed(0..program_data.world_datas.main_index_count, 0, 0..1);
	drop(main_pass_handle);
	
	program_data.render_context.command_queue.submit(std::iter::once(encoder.finish()));
	program_data.window.pre_present_notify();
	output.present();
	
	StdResult::Ok(())
}

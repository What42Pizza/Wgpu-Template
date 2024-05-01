use crate::prelude::*;
use std::ops::Range;



pub fn render(output: &wgpu::SurfaceTexture, program_data: &mut ProgramData) -> StdResult<(), wgpu::SurfaceError> {
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
		depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
			view: &program_data.assets.depth.view,
			depth_ops: Some(wgpu::Operations {
				load: wgpu::LoadOp::Clear(1.0),
				store: wgpu::StoreOp::Store,
			}),
			stencil_ops: None,
		}),
		occlusion_query_set: None,
		timestamp_writes: None,
	});
	main_pass_handle.set_pipeline(&program_data.render_pipelines.main);
	main_pass_handle.set_bind_group(0, &program_data.bindings.camera_group, &[]);
	main_pass_handle.set_bind_group(1, &program_data.bindings.happy_tree_group, &[]);
	//main_pass_handle.set_vertex_buffer(0, program_data.world_data.main_vertices.slice(..));
	//main_pass_handle.set_vertex_buffer(1, program_data.world_data.main_instances_buffer.slice(..));
	//main_pass_handle.set_index_buffer(program_data.world_data.main_indices.slice(..), wgpu::IndexFormat::Uint16);
	//main_pass_handle.draw_indexed(0..program_data.world_data.main_index_count, 0, 0..program_data.world_data.main_instances.len() as u32);
	main_pass_handle.set_vertex_buffer(1, program_data.world_data.main_instances_buffer.slice(..));
	draw_mesh_instanced(&mut main_pass_handle, &program_data.assets.test_model.meshes[0], 0..program_data.world_data.main_instances.len() as u32);
	drop(main_pass_handle);
	
	program_data.render_context.command_queue.submit(std::iter::once(encoder.finish()));
	
	StdResult::Ok(())
}



fn draw_mesh_instanced<'a, 'b> (
	render_pass: &mut wgpu::RenderPass<'a>,
	mesh: &'b Mesh,
	instances: Range<u32>,
) where
	'b: 'a
{
	render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
	render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
	render_pass.draw_indexed(0..mesh.num_elements, 0, instances);
}

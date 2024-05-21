use crate::prelude::*;



pub fn render(output: &wgpu::SurfaceTexture, program_data: &mut ProgramData) {
	
	let frustum_planes = get_frustum_planes(&program_data.camera_data, program_data.render_context.aspect_ratio);
	let visible_models_list = get_visible_models(
		&program_data.example_model_instance_datas,
		program_data.render_assets.example_models.bounding_radius,
		&frustum_planes
	);
	
	update_gpu_buffers(program_data, &visible_models_list);
	
	let output_view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
	let main_tex_view = &program_data.render_assets.main_tex_view;
	let encoder_descriptor = wgpu::CommandEncoderDescriptor {label: None};
	let mut encoder = program_data.render_context.device.create_command_encoder(&encoder_descriptor);
	
	render_shadow_caster_pipeline(program_data, &mut encoder);
	render_models_pipeline(program_data, &mut encoder, &main_tex_view);
	render_skybox_pipeline(program_data, &mut encoder, &main_tex_view); // HELP: it's better to have this at the end so that only the necessary pixels are rendered
	render_color_correction_pipeline(program_data, &mut encoder, &output_view);
	
	program_data.render_context.command_queue.submit(std::iter::once(encoder.finish()));
}



// this is an implementation of frustum culling based on: https://learnopengl.com/Guest-Articles/2021/Scene/Frustum-Culling
pub fn get_visible_models(instance_datas: &[InstanceData], bounding_radius: f32, frustum_planes: &[(glam::Vec3, f32); 5]) -> Vec<usize> {
	let mut output = Vec::with_capacity(instance_datas.len());
	for (i, instance) in instance_datas.iter().enumerate() {
		if model_is_visible(&instance.pos, bounding_radius, &frustum_planes) {
			output.push(i);
		}
	}
	output
}

pub fn get_frustum_planes(camera: &CameraData, aspect_ratio: f32) -> [(glam::Vec3, f32); 5] {
	let forward = glam::Vec3::new(
		camera.rot_xz.cos() * camera.rot_y.cos(),
		camera.rot_y.sin(),
		camera.rot_xz.sin() * camera.rot_y.cos(),
	).normalize();
	
	let up_dir = glam::Vec3::Y;
	let right_dir = forward.cross(up_dir).normalize();
	let up_dir = forward.cross(-right_dir).normalize();
	const FOV_MULT: f32 = 1.0; // lower this to see the culling work
	let half_height = (camera.fov_radians * FOV_MULT / 2.0).tan() * camera.near;
	let half_width = half_height * aspect_ratio;
	let near_plane_center = camera.pos + forward * camera.near;
	
	let left_plane = {
		let point_on_plane = near_plane_center - right_dir * half_width;
		let normal = -up_dir.cross(point_on_plane - camera.pos).normalize();
		let dist = point_on_plane.dot(normal);
		(normal, dist)
	};
	
	let right_plane = {
		let point_on_plane = near_plane_center + right_dir * half_width;
		let normal = up_dir.cross(point_on_plane - camera.pos).normalize();
		let dist = point_on_plane.dot(normal);
		(normal, dist)
	};
	
	let top_plane = {
		let point_on_plane = near_plane_center + up_dir * half_height;
		let normal = -right_dir.cross(point_on_plane - camera.pos).normalize();
		let dist = point_on_plane.dot(normal);
		(normal, dist)
	};
	
	let bottom_plane = {
		let point_on_plane = near_plane_center - up_dir * half_height;
		let normal = right_dir.cross(point_on_plane - camera.pos).normalize();
		let dist = point_on_plane.dot(normal);
		(normal, dist)
	};
	
	//let near_plane = { // there's not really any point in checking this plane but you still can if you just want to
	//	let normal = forward;
	//	let point_on_plane = camera.pos + forward * camera.near;
	//	let dist = point_on_plane.dot(forward);
	//	(normal, dist)
	//};
	
	let far_plane = { // you might be able to remove this one too, I have it last b/c it should be the least likely to trigger removal
		let normal = -forward;
		let point_on_plane = camera.pos + forward * camera.far;
		let dist = point_on_plane.dot(normal);
		(normal, dist)
	};
	
	[far_plane, left_plane, right_plane, top_plane, bottom_plane]
}

pub fn model_is_visible(pos: &glam::Vec3, bounding_radius: f32, frustum_planes: &[(glam::Vec3, f32); 5]) -> bool {
	is_sphere_past_plane(pos, bounding_radius, &frustum_planes[0].0, frustum_planes[0].1)
		&& is_sphere_past_plane(pos, bounding_radius, &frustum_planes[1].0, frustum_planes[1].1)
		&& is_sphere_past_plane(pos, bounding_radius, &frustum_planes[2].0, frustum_planes[2].1)
		&& is_sphere_past_plane(pos, bounding_radius, &frustum_planes[3].0, frustum_planes[3].1)
		&& is_sphere_past_plane(pos, bounding_radius, &frustum_planes[4].0, frustum_planes[4].1)
}

pub fn is_sphere_past_plane(pos: &glam::Vec3, radius: f32, plane_normal: &glam::Vec3, plane_dist: f32) -> bool {
	plane_normal.dot(*pos) > plane_dist - radius
}



pub fn update_gpu_buffers(program_data: &mut ProgramData, visible_models: &[usize]) {
	
	// culled_instances_buffer
	let mut new_model_instances_data = Vec::with_capacity(visible_models.len());
	for index in visible_models {
		new_model_instances_data.push(program_data.example_model_instance_datas[*index].to_raw())
	}
	program_data.render_context.command_queue.write_buffer(
		&program_data.render_assets.example_models.culled_instances_buffer,
		0,
		bytemuck::cast_slice(&new_model_instances_data),
	);
	program_data.render_assets.example_models.culled_instances_count = visible_models.len() as u32;
	
	// camera.buffer
	let camera_gpu_data = program_data.camera_data.build_gpu_data(program_data.render_context.aspect_ratio);
	program_data.render_context.command_queue.write_buffer(
		&program_data.render_assets.camera.buffer,
		0,
		bytemuck::cast_slice(&camera_gpu_data),
	);
	
	// shadow_caster.proj_mat_buffer
	let shadow_caster_gpu_data = program_data.shadow_caster_data.build_gpu_data(program_data.camera_data.pos);
	program_data.render_context.command_queue.write_buffer(
		&program_data.render_assets.shadow_caster.proj_mat_buffer,
		0,
		bytemuck::cast_slice(&shadow_caster_gpu_data),
	);
	
}





pub fn render_shadow_caster_pipeline(program_data: &ProgramData, encoder: &mut wgpu::CommandEncoder) {
	let render_assets = &program_data.render_assets;
	
	// I've tried to move these RenderPassDescriptor-s to `load_layouts.rs`, but the complexity required just isn't worth it
	let mut shadow_caster_pass_handle = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
		label: Some("shadow_caster_render_pass"),
		color_attachments: &[],
		depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
			view: &render_assets.shadow_caster.depth_tex_view,
			depth_ops: Some(wgpu::Operations {
				load: wgpu::LoadOp::Clear (1.0),
				store: wgpu::StoreOp::Store,
			}),
			stencil_ops: None,
		}),
		occlusion_query_set: None,
		timestamp_writes: None,
	});
	
	shadow_caster_pass_handle.set_pipeline(&program_data.render_layouts.shadow_caster_pipeline);
	shadow_caster_pass_handle.set_bind_group(0, &program_data.render_bindings.shadow_caster_bind_0, &[]);
	
	for mesh in &render_assets.example_models.meshes {
		shadow_caster_pass_handle.set_vertex_buffer(0, mesh.basic_vertex_buffer.slice(..));
		shadow_caster_pass_handle.set_vertex_buffer(1, render_assets.example_models.total_instances_buffer.slice(..));
		shadow_caster_pass_handle.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
		shadow_caster_pass_handle.draw_indexed(0..mesh.index_count, 0, 0..render_assets.example_models.total_instances_count);
	}
	
}





pub fn render_models_pipeline(program_data: &ProgramData, encoder: &mut wgpu::CommandEncoder, main_tex_view: &wgpu::TextureView) {
	let render_assets = &program_data.render_assets;
	
	let mut models_pass_handle = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
		label: Some("models_render_pass"),
		color_attachments: &[Some(wgpu::RenderPassColorAttachment {
			view: main_tex_view,
			resolve_target: None,
			ops: wgpu::Operations {
				load: wgpu::LoadOp::Clear (wgpu::Color {
					r: 0.1,
					g: 0.2,
					b: 0.3,
					a: 1.0,
				}),
				store: wgpu::StoreOp::Store,
			},
		})],
		depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
			view: &render_assets.depth.view,
			depth_ops: Some(wgpu::Operations {
				load: wgpu::LoadOp::Clear (1.0),
				store: wgpu::StoreOp::Store,
			}),
			stencil_ops: None,
		}),
		occlusion_query_set: None,
		timestamp_writes: None,
	});
	
	models_pass_handle.set_pipeline(&program_data.render_layouts.models_pipeline);
	models_pass_handle.set_bind_group(0, &program_data.render_bindings.models_bind_0, &[]);
	
	for (i, mesh) in program_data.render_assets.example_models.meshes.iter().enumerate() {
		models_pass_handle.set_bind_group(1, &program_data.render_bindings.example_models_bind_1s[i], &[]);
		models_pass_handle.set_vertex_buffer(0, mesh.basic_vertex_buffer.slice(..));
		models_pass_handle.set_vertex_buffer(1, mesh.extended_vertex_buffer.slice(..));
		models_pass_handle.set_vertex_buffer(2, render_assets.example_models.culled_instances_buffer.slice(..));
		models_pass_handle.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
		models_pass_handle.draw_indexed(0..mesh.index_count, 0, 0..render_assets.example_models.culled_instances_count);
	}
	
}





pub fn render_skybox_pipeline(program_data: &ProgramData, encoder: &mut wgpu::CommandEncoder, main_tex_view: &wgpu::TextureView) {
	let render_assets = &program_data.render_assets;
	
	let mut skybox_pass_handle = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
		label: Some("skybox_render_pass"),
		color_attachments: &[Some(wgpu::RenderPassColorAttachment {
			view: main_tex_view,
			resolve_target: None,
			ops: wgpu::Operations {
				load: wgpu::LoadOp::Load,
				store: wgpu::StoreOp::Store,
			},
		})],
		depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
			view: &render_assets.depth.view,
			depth_ops: Some(wgpu::Operations {
				load: wgpu::LoadOp::Load,
				store: wgpu::StoreOp::Store,
			}),
			stencil_ops: None,
		}),
		occlusion_query_set: None,
		timestamp_writes: None,
	});
	
	skybox_pass_handle.set_pipeline(&program_data.render_layouts.skybox_pipeline);
	skybox_pass_handle.set_bind_group(0, &program_data.render_bindings.skybox_bind_0, &[]);
	
	skybox_pass_handle.draw(0..3, 0..1)
	
}





pub fn render_color_correction_pipeline(program_data: &ProgramData, encoder: &mut wgpu::CommandEncoder, output_view: &wgpu::TextureView) {
	let render_assets = &program_data.render_assets;
	
	let mut skybox_pass_handle = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
		label: Some("color_correction_render_pass"),
		color_attachments: &[Some(wgpu::RenderPassColorAttachment {
			view: output_view,
			resolve_target: None,
			ops: wgpu::Operations {
				load: wgpu::LoadOp::Load,
				store: wgpu::StoreOp::Store,
			},
		})],
		depth_stencil_attachment: None,
		occlusion_query_set: None,
		timestamp_writes: None,
	});
	
	skybox_pass_handle.set_pipeline(&program_data.render_layouts.color_correction_pipeline);
	skybox_pass_handle.set_bind_group(0, &program_data.render_bindings.color_correction_bind_0, &[]);
	
	skybox_pass_handle.draw(0..3, 0..1)
	
}

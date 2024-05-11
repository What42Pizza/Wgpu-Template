use crate::prelude::*;



pub fn load_render_bindings(render_context: &RenderContextData, render_layouts: &RenderLayouts, render_assets: &RenderAssets) -> Result<RenderBindings> {
	
	
	
	let shadow_caster_bind_0 = render_context.device.create_bind_group(&wgpu::BindGroupDescriptor {
		label: Some("shadow_caster_bind_0"),
		layout: &render_layouts.shadow_caster_bind_0_layout,
		entries: &[
			wgpu::BindGroupEntry {
				binding: 0,
				resource: render_assets.shadow_caster.proj_mat_buffer.as_entire_binding(),
			},
		],
	});
	
	
	
	let models_bind_0 = render_context.device.create_bind_group(&wgpu::BindGroupDescriptor {
		label: Some("models_bind_0"),
		layout: &render_layouts.models_bind_0_layout,
		entries: &[
			wgpu::BindGroupEntry {
				binding: 0,
				resource: render_assets.camera.buffer.as_entire_binding(),
			},
			wgpu::BindGroupEntry {
				binding: 1,
				resource: render_assets.shadow_caster.proj_mat_buffer.as_entire_binding(),
			},
			wgpu::BindGroupEntry {
				binding: 2,
				resource: wgpu::BindingResource::Sampler (&render_assets.default_sampler),
			},
			wgpu::BindGroupEntry {
				binding: 3,
				resource: wgpu::BindingResource::TextureView (&render_assets.shadow_caster.depth_tex_view),
			},
			wgpu::BindGroupEntry {
				binding: 4,
				resource: wgpu::BindingResource::Sampler (&render_assets.shadow_caster.depth_sampler),
			},
			wgpu::BindGroupEntry {
				binding: 5,
				resource: wgpu::BindingResource::Sampler (&render_assets.shadow_caster.debug_depth_sampler),
			},
		],
	});
	
	let mut example_models_bind_1s = vec!();
	for (i, mesh) in render_assets.example_models.meshes.iter().enumerate() {
		let material_view = &render_assets.materials_storage.list_2d[mesh.material_id].view;
		let bind = render_context.device.create_bind_group(&wgpu::BindGroupDescriptor {
			label: Some(&format!("example_model_mesh_{i}_bind_1")),
			layout: &render_layouts.models_bind_1_layout,
			entries: &[
				wgpu::BindGroupEntry {
					binding: 0,
					resource: wgpu::BindingResource::TextureView (material_view),
				},
			],
		});
		example_models_bind_1s.push(bind);
	}
	
	
	
	let skybox_view = &render_assets.materials_storage.list_cube[render_assets.skybox_material_id].view;
	let skybox_bind_0 = render_context.device.create_bind_group(&wgpu::BindGroupDescriptor {
		label: Some("skybox_bind_0"),
		layout: &render_layouts.skybox_bind_0_layout,
		entries: &[
			wgpu::BindGroupEntry {
				binding: 0,
				resource: render_assets.camera.buffer.as_entire_binding(),
			},
			wgpu::BindGroupEntry {
				binding: 1,
				resource: wgpu::BindingResource::TextureView (&skybox_view),
			},
			wgpu::BindGroupEntry {
				binding: 2,
				resource: wgpu::BindingResource::Sampler (&render_assets.skybox_sampler),
			},
		],
	});
	
	
	
	Ok(RenderBindings {
		
		shadow_caster_bind_0,
		
		models_bind_0,
		example_models_bind_1s,
		
		skybox_bind_0,
		
	})
}

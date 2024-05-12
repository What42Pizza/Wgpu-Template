use crate::prelude::*;



pub fn load_render_bindings(render_context: &RenderContextData, render_layouts: &RenderLayouts, render_assets: &RenderAssets) -> Result<RenderBindings> {
	
	
	
	let skybox_view = &render_assets.materials_storage.list_cube[render_assets.skybox_material_id].view;
	
	let bind_0 = render_context.device.create_bind_group(&wgpu::BindGroupDescriptor {
		label: Some("bind_0"),
		layout: &render_layouts.bind_0_layout,
		entries: &[
			
			// basics	
			wgpu::BindGroupEntry { // camera: proj_vew_mat, inv_proj_mat, view_mat
				binding: 0,
				resource: render_assets.camera.buffer.as_entire_binding(),
			},
			wgpu::BindGroupEntry { // models: sampler
				binding: 1,
				resource: wgpu::BindingResource::Sampler (&render_assets.default_sampler),
			},
			
			// shadow_caster
			wgpu::BindGroupEntry { // shadow_caster: proj_mat
				binding: 2,
				resource: render_assets.shadow_caster.proj_mat_buffer.as_entire_binding(),
			},
			wgpu::BindGroupEntry { // shadow_caster: tex_view
				binding: 3,
				resource: wgpu::BindingResource::TextureView (&render_assets.shadow_caster.depth_tex_view),
			},
			wgpu::BindGroupEntry { // shadow_caster: sampler
				binding: 4,
				resource: wgpu::BindingResource::Sampler (&render_assets.shadow_caster.depth_sampler),
			},
			wgpu::BindGroupEntry { // shadow_caster: debug_sampler
				binding: 5,
				resource: wgpu::BindingResource::Sampler (&render_assets.shadow_caster.debug_depth_sampler),
			},
			
			// skybox
			wgpu::BindGroupEntry { // skybox: tex_view
				binding: 6,
				resource: wgpu::BindingResource::TextureView (skybox_view),
			},
			wgpu::BindGroupEntry { // skybox: sampler
				binding: 7,
				resource: wgpu::BindingResource::Sampler (&render_assets.skybox_sampler),
			},
			
		]
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
	
	
	
	Ok(RenderBindings {
		
		bind_0,
		
		example_models_bind_1s,
		
	})
}

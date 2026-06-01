use crate::*;



pub struct ModelsShadowmapLayouts {
	pub render_pipeline: wgpu::RenderPipeline,
	pub bind_group_0: wgpu::BindGroupLayout,
}

pub struct ModelsShadowmapBindings (wgpu::BindGroup);



pub fn load_layout(render_context: &RenderContext) -> Result<ModelsShadowmapLayouts> {
	
	let models_shadowmap_shader_path = utils::get_program_file_path("shaders/models_shadowmap.wgsl");
	let models_shadowmap_shader_source = fs_read_to_string(&models_shadowmap_shader_path)?;
	let models_shadowmap_shader = render_context.gpu_device.create_shader_module(wgpu::ShaderModuleDescriptor {
		label: Some("models_shadowmap_shader_module"),
		source: wgpu::ShaderSource::Wgsl(models_shadowmap_shader_source.into()),
	});
	
	let bind_group_0 = render_context.gpu_device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
		label: Some("models_shadowmap_bind_0_layout"),
		entries: &[
			wgpu::BindGroupLayoutEntry { // models_shadowmap: proj_mat
				binding: 0,
				visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
				ty: wgpu::BindingType::Buffer {
					ty: wgpu::BufferBindingType::Uniform,
					has_dynamic_offset: false,
					min_binding_size: None,
				},
				count: None,
			},
		]
	});
	
	let pipeline_layout = render_context.gpu_device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
		label: Some("models_shadowmap_pipeline_layout"),
		bind_group_layouts: &[
			Some(&bind_group_0),
		],
		immediate_size: 0,
	});
	let render_pipeline = render_context.gpu_device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
		label: Some("models_shadowmap_pipeline"),
		layout: Some(&pipeline_layout),
		vertex: wgpu::VertexState {
			module: &models_shadowmap_shader,
			entry_point: Some("vs_main"),
			buffers: &[
				BasicVertexData::get_layout(),
				RawInstanceData::get_layout()
			],
			compilation_options: wgpu::PipelineCompilationOptions::default(),
		},
		fragment: None,
		primitive: wgpu::PrimitiveState {
			topology: wgpu::PrimitiveTopology::TriangleList,
			strip_index_format: None,
			front_face: wgpu::FrontFace::Ccw,
			cull_mode: Some(wgpu::Face::Front), // idk why this needs to be different from the models pipeline
			polygon_mode: wgpu::PolygonMode::Fill,
			unclipped_depth: false,
			conservative: false,
		},
		depth_stencil: Some(wgpu::DepthStencilState {
			format: wgpu::TextureFormat::Depth32Float,
			depth_write_enabled: Some(true),
			depth_compare: Some(wgpu::CompareFunction::LessEqual),
			stencil: wgpu::StencilState::default(),
			bias: wgpu::DepthBiasState {
				constant: 2, // HELP: corresponds to bilinear filtering
				slope_scale: 2.0,
				clamp: 0.0,
			},
		}),
		multisample: wgpu::MultisampleState {
			count: 1,
			mask: u64::MAX,
			alpha_to_coverage_enabled: false,
		},
		multiview_mask: None,
		cache: None,
	});
	
	Ok(ModelsShadowmapLayouts {
		render_pipeline,
		bind_group_0,
	})
}



pub fn load_bindings(render_context: &RenderContext, render_layouts: &ModelsShadowmapLayouts, render_assets: &RenderAssets) -> ModelsShadowmapBindings {
	
	let bind_group_0 = render_context.gpu_device.create_bind_group(&wgpu::BindGroupDescriptor {
		label: Some("models_shadowmap_bind_0"),
		layout: &render_layouts.bind_group_0,
		entries: &[
			wgpu::BindGroupEntry {
				binding: 0,
				resource: render_assets.shadowmap.proj_mat_buffer.as_entire_binding(),
			},
		],
	});
	
	ModelsShadowmapBindings (bind_group_0)
}



pub fn render(layouts: &ModelsShadowmapLayouts, assets: &RenderAssets, bindings: &ModelsShadowmapBindings, encoder: &mut wgpu::CommandEncoder) {
	
	// I've tried to move these RenderPassDescriptor-s to `load_layouts.rs`, but the complexity required just isn't worth it
	let mut shadow_caster_pass_handle = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
		label: Some("shadow_caster_render_pass"),
		color_attachments: &[],
		depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
			view: &assets.shadowmap.depth_tex_view,
			depth_ops: Some(wgpu::Operations {
				load: wgpu::LoadOp::Clear (1.0),
				store: wgpu::StoreOp::Store,
			}),
			stencil_ops: None,
		}),
		multiview_mask: None,
		occlusion_query_set: None,
		timestamp_writes: None,
	});
	
	shadow_caster_pass_handle.set_pipeline(&layouts.render_pipeline);
	shadow_caster_pass_handle.set_bind_group(0, &bindings.0, &[]);
	
	for mesh in &assets.example_models.meshes {
		shadow_caster_pass_handle.set_vertex_buffer(0, mesh.basic_vertex_buffer.slice(..));
		shadow_caster_pass_handle.set_vertex_buffer(1, assets.example_models.total_instances_buffer.slice(..));
		shadow_caster_pass_handle.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
		shadow_caster_pass_handle.draw_indexed(0..mesh.index_count, 0, 0..assets.example_models.total_instances_count);
	}
	
}

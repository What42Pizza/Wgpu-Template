use crate::*;



pub struct ModelsMainLayouts {
	pub render_pipeline: wgpu::RenderPipeline,
	pub bind_group_0: wgpu::BindGroupLayout,
	pub bind_group_1: wgpu::BindGroupLayout,
}

pub struct ModelsMainBindings (wgpu::BindGroup, Vec<wgpu::BindGroup>);



pub fn load_layout(render_context: &RenderContext) -> Result<ModelsMainLayouts> {
	
	let models_shader_path = utils::get_program_file_path("shaders/models_main.wgsl");
	let models_shader_source = fs_read_to_string(&models_shader_path)?;
	let models_shader = render_context.gpu_device.create_shader_module(wgpu::ShaderModuleDescriptor {
		label: Some("models_shader_module"),
		source: wgpu::ShaderSource::Wgsl(models_shader_source.into()),
	});
	
	let bind_group_0 = render_context.gpu_device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
		label: Some("models_bind_0_layout"),
		entries: &[
			wgpu::BindGroupLayoutEntry { // camera: proj_view_mat, inv_proj_mat, view_mat
				binding: 0,
				visibility: wgpu::ShaderStages::VERTEX,
				ty: wgpu::BindingType::Buffer {
					ty: wgpu::BufferBindingType::Uniform,
					has_dynamic_offset: false,
					min_binding_size: None,
				},
				count: None,
			},
			wgpu::BindGroupLayoutEntry { // shadow_caster: proj_mat
				binding: 1,
				visibility: wgpu::ShaderStages::FRAGMENT,
				ty: wgpu::BindingType::Buffer {
					ty: wgpu::BufferBindingType::Uniform,
					has_dynamic_offset: false,
					min_binding_size: None,
				},
				count: None,
			},
			wgpu::BindGroupLayoutEntry { // material: sampler
				binding: 2,
				visibility: wgpu::ShaderStages::FRAGMENT,
				ty: wgpu::BindingType::Sampler (wgpu::SamplerBindingType::Filtering),
				count: None,
			},
			wgpu::BindGroupLayoutEntry { // shadowmap: texture
				binding: 3,
				visibility: wgpu::ShaderStages::FRAGMENT,
				ty: wgpu::BindingType::Texture {
					multisampled: false,
					view_dimension: wgpu::TextureViewDimension::D2,
					sample_type: wgpu::TextureSampleType::Depth,
				},
				count: None,
			},
			wgpu::BindGroupLayoutEntry { // shadowmap: sampler
				binding: 4,
				visibility: wgpu::ShaderStages::FRAGMENT,
				ty: wgpu::BindingType::Sampler (wgpu::SamplerBindingType::Comparison),
				count: None,
			},
		]
	});
	
	let bind_group_1 = render_context.gpu_device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
		label: Some("models_bind_1_layout"),
		entries: &[
			wgpu::BindGroupLayoutEntry { // material: view
				binding: 0,
				visibility: wgpu::ShaderStages::FRAGMENT,
				ty: wgpu::BindingType::Texture {
					multisampled: false,
					view_dimension: wgpu::TextureViewDimension::D2,
					sample_type: wgpu::TextureSampleType::Float { filterable: true },
				},
				count: None,
			},
		],
	});
	
	let pipeline_layout = render_context.gpu_device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
		label: Some("models_render_pipeline_layout"),
		bind_group_layouts: &[
			Some(&bind_group_0),
			Some(&bind_group_1),
		],
		immediate_size: 0,
	});
	let render_pipeline = render_context.gpu_device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
		label: Some("models_render_pipeline"),
		layout: Some(&pipeline_layout),
		vertex: wgpu::VertexState {
			module: &models_shader,
			entry_point: Some("vs_main"),
			buffers: &[
				BasicVertexData::get_layout(),
				ExtendedVertexData::get_layout(),
				RawInstanceData::get_layout(),
			],
			compilation_options: wgpu::PipelineCompilationOptions::default(),
		},
		fragment: Some(wgpu::FragmentState {
			module: &models_shader,
			entry_point: Some("fs_main"),
			targets: &[Some(wgpu::ColorTargetState {
				format: render_context.surface_config.format,
				blend: Some(wgpu::BlendState::REPLACE),
				write_mask: wgpu::ColorWrites::ALL,
			})],
			compilation_options: wgpu::PipelineCompilationOptions::default(),
		}),
		primitive: wgpu::PrimitiveState {
			topology: wgpu::PrimitiveTopology::TriangleList,
			strip_index_format: None,
			front_face: wgpu::FrontFace::Ccw,
			cull_mode: Some(wgpu::Face::Back),
			polygon_mode: wgpu::PolygonMode::Fill,
			unclipped_depth: false,
			conservative: false,
		},
		depth_stencil: Some(wgpu::DepthStencilState {
			format: wgpu::TextureFormat::Depth32Float,
			depth_write_enabled: Some(true),
			depth_compare: Some(wgpu::CompareFunction::Less),
			stencil: wgpu::StencilState::default(),
			bias: wgpu::DepthBiasState::default(),
		}),
		multisample: wgpu::MultisampleState {
			count: 1,
			mask: u64::MAX,
			alpha_to_coverage_enabled: false,
		},
		multiview_mask: None,
		cache: None,
	});
	
	Ok(ModelsMainLayouts {
		render_pipeline,
		bind_group_0,
		bind_group_1,
	})
}



pub fn load_bindings(render_context: &RenderContext, render_layouts: &ModelsMainLayouts, render_assets: &RenderAssets) -> ModelsMainBindings {
	
	let bind_group_0 = render_context.gpu_device.create_bind_group(&wgpu::BindGroupDescriptor {
		label: Some("models_bind_0"),
		layout: &render_layouts.bind_group_0,
		entries: &[
			wgpu::BindGroupEntry {
				binding: 0,
				resource: render_assets.camera_data.as_entire_binding(),
			},
			wgpu::BindGroupEntry {
				binding: 1,
				resource: render_assets.shadowmap.proj_mat_buffer.as_entire_binding(),
			},
			wgpu::BindGroupEntry {
				binding: 2,
				resource: wgpu::BindingResource::Sampler (&render_assets.default_sampler),
			},
			wgpu::BindGroupEntry {
				binding: 3,
				resource: wgpu::BindingResource::TextureView (&render_assets.shadowmap.depth_tex_view),
			},
			wgpu::BindGroupEntry {
				binding: 4,
				resource: wgpu::BindingResource::Sampler (&render_assets.shadowmap.depth_sampler),
			},
		],
	});
	
	let mut bind_group_1_vec = vec!();
	for (i, mesh) in render_assets.example_models.meshes.iter().enumerate() {
		let material_view = &render_assets.materials_storage.list_2d[mesh.material_id].view;
		let bind = render_context.gpu_device.create_bind_group(&wgpu::BindGroupDescriptor {
			label: Some(&format!("example_model_mesh_{i}_bind_1")),
			layout: &render_layouts.bind_group_1,
			entries: &[
				wgpu::BindGroupEntry {
					binding: 0,
					resource: wgpu::BindingResource::TextureView (material_view),
				},
			],
		});
		bind_group_1_vec.push(bind);
	}
	
	ModelsMainBindings (bind_group_0, bind_group_1_vec)
}



pub fn render(layouts: &ModelsMainLayouts, assets: &RenderAssets, bindings: &ModelsMainBindings, encoder: &mut wgpu::CommandEncoder) {
	
	let mut models_pass_handle = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
		label: Some("models_render_pass"),
		color_attachments: &[Some(wgpu::RenderPassColorAttachment {
			view: &assets.main_tex_view,
			depth_slice: None,
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
			view: &assets.main_depth_view,
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
	
	models_pass_handle.set_pipeline(&layouts.render_pipeline);
	models_pass_handle.set_bind_group(0, &bindings.0, &[]);
	
	for (i, mesh) in assets.example_models.meshes.iter().enumerate() {
		models_pass_handle.set_bind_group(1, &bindings.1[i], &[]);
		models_pass_handle.set_vertex_buffer(0, mesh.basic_vertex_buffer.slice(..));
		models_pass_handle.set_vertex_buffer(1, mesh.extended_vertex_buffer.slice(..));
		models_pass_handle.set_vertex_buffer(2, assets.example_models.culled_instances_buffer.slice(..));
		models_pass_handle.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
		models_pass_handle.draw_indexed(0..mesh.index_count, 0, 0..assets.example_models.culled_instances_count);
	}
	
}

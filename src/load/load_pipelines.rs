use crate::prelude::*;



pub fn load_render_pipelines(render_context: &RenderContextData, render_assets: &RenderAssets, binding_1_layout: wgpu::BindGroupLayout) -> Result<RenderPipelines> {
	
	let (
		shadowmap_pipeline,
		shadowmap_bind_0_layout,
		shadowmap_bind_0,
	) =
		load_shadowmap_render_pipeline(
			render_context,
			&render_assets.shadow_caster,
		)
		.context("Failed to load shadowmap render pipeline.")?;
	
	let (
		models_pipeline,
		models_bind_0_layout,
		models_bind_0,
		//models_bind_1_layout
	) =
		load_models_render_pipeline(
			render_context,
			&render_assets.camera,
			&render_assets.shadow_caster,
			&binding_1_layout,
		)
		.context("Failed to load models render pipeline.")?;
	
	let (
		skybox_pipeline,
		skybox_bind_0_layout,
		skybox_bind_0,
	) =
		load_skybox_render_pipeline(
			render_context,
			&render_assets.camera,
			&render_assets.materials_storage.list_cube[render_assets.skybox_material_id].view,
		)
		.context("Failed to load skybox render pipeline.")?;
	
	Ok(RenderPipelines {
		
		shadowmap_pipeline,
		shadowmap_bind_0_layout,
		shadowmap_bind_0,
		
		models_pipeline,
		models_bind_0_layout,
		models_bind_0,
		models_bind_1_layout: binding_1_layout,
		
		skybox_pipeline,
		skybox_bind_0_layout,
		skybox_bind_0,
		
	})
}





pub fn load_shadowmap_render_pipeline(
	render_context: &RenderContextData,
	shadow_caster_data: &ShadowCasterRenderData,
) -> Result<(
	wgpu::RenderPipeline,
	wgpu::BindGroupLayout,
	wgpu::BindGroup,
)> {
	
	let shader_path = utils::get_program_file_path("shaders/shadowmap.wgsl");
	let shader_source = fs::read_to_string(&shader_path).add_path_to_error(&shader_path)?;
	let shader = render_context.device.create_shader_module(wgpu::ShaderModuleDescriptor {
		label: Some("Shadowmap Render Pipeline"),
		source: wgpu::ShaderSource::Wgsl(shader_source.into()),
	});
	
	let (bind_group_0_layout, bind_group_0) = get_full_bind_group_data(Some("shadow_caster_bind_group_0"), &render_context.device, vec!(
		(
			// shadow_caster: proj_mat
			wgpu::BindGroupLayoutEntry {
				binding: 0,
				visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
				ty: wgpu::BindingType::Buffer {
					ty: wgpu::BufferBindingType::Uniform,
					has_dynamic_offset: false,
					min_binding_size: None,
				},
				count: None,
			},
			shadow_caster_data.proj_mat_buffer.as_entire_binding(),
		),
	));
	
	let pipeline_layout = render_context.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
		label: Some("Shadowmap Render Pipeline"),
		bind_group_layouts: &[
			&bind_group_0_layout,
		],
		push_constant_ranges: &[],
	});
	let pipeline = render_context.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
		label: Some("Shadowmap Render Pipeline"),
		layout: Some(&pipeline_layout),
		vertex: wgpu::VertexState {
			module: &shader,
			entry_point: "vs_main",
			buffers: &[
				BasicVertexData::get_layout(),
				ExtendedVertexData::get_layout(),
				RawInstanceData::get_layout()
			],
			compilation_options: wgpu::PipelineCompilationOptions::default(),
		},
		fragment: None,
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
			depth_write_enabled: true,
			depth_compare: wgpu::CompareFunction::LessEqual,
			stencil: wgpu::StencilState::default(),
			bias: wgpu::DepthBiasState {
				constant: 2, // corresponds to bilinear filtering
				slope_scale: 2.0,
				clamp: 0.0,
			},
		}),
		multisample: wgpu::MultisampleState {
			count: 1,
			mask: !0u64,
			alpha_to_coverage_enabled: false,
		},
		multiview: None,
	});
	
	Ok((
		pipeline,
		bind_group_0_layout,
		bind_group_0,
	))
}





pub fn load_models_render_pipeline(
	render_context: &RenderContextData,
	camera_data: &CameraRenderData,
	shadow_caster_data: &ShadowCasterRenderData,
	binding_1_layout: &wgpu::BindGroupLayout,
) -> Result<(
	wgpu::RenderPipeline,
	wgpu::BindGroupLayout,
	wgpu::BindGroup,
	//wgpu::BindGroupLayout,
)> {
	
	let shader_path = utils::get_program_file_path("shaders/models.wgsl");
	let shader_source = fs::read_to_string(&shader_path).add_path_to_error(&shader_path)?;
	let shader = render_context.device.create_shader_module(wgpu::ShaderModuleDescriptor {
		label: Some("Models Render Pipeline"),
		source: wgpu::ShaderSource::Wgsl(shader_source.into()),
	});
	
	let sampler = render_context.device.create_sampler(&wgpu::SamplerDescriptor {
		address_mode_u: wgpu::AddressMode::ClampToEdge,
		address_mode_v: wgpu::AddressMode::ClampToEdge,
		address_mode_w: wgpu::AddressMode::ClampToEdge,
		mag_filter: wgpu::FilterMode::Linear,
		min_filter: wgpu::FilterMode::Nearest,
		mipmap_filter: wgpu::FilterMode::Nearest,
		..Default::default()
	});
	let depth_tex_sampler = render_context.device.create_sampler(&wgpu::SamplerDescriptor {
		address_mode_u: wgpu::AddressMode::ClampToEdge,
		address_mode_v: wgpu::AddressMode::ClampToEdge,
		address_mode_w: wgpu::AddressMode::ClampToEdge,
		mag_filter: wgpu::FilterMode::Linear,
		min_filter: wgpu::FilterMode::Linear,
		mipmap_filter: wgpu::FilterMode::Nearest,
		compare: Some(wgpu::CompareFunction::LessEqual),
		..Default::default()
	});
	let debug_depth_tex_sampler = render_context.device.create_sampler(&wgpu::SamplerDescriptor {
		address_mode_u: wgpu::AddressMode::ClampToEdge,
		address_mode_v: wgpu::AddressMode::ClampToEdge,
		address_mode_w: wgpu::AddressMode::ClampToEdge,
		mag_filter: wgpu::FilterMode::Linear,
		min_filter: wgpu::FilterMode::Linear,
		mipmap_filter: wgpu::FilterMode::Nearest,
		..Default::default()
	});
	let (bind_group_0_layout, bind_group_0) = get_full_bind_group_data(Some("models_bind_group_0"), &render_context.device, vec!(
		(
			// camera: proj_view_mat, inv_proj_mat, view_mat
			wgpu::BindGroupLayoutEntry {
				binding: 0,
				visibility: wgpu::ShaderStages::VERTEX,
				ty: wgpu::BindingType::Buffer {
					ty: wgpu::BufferBindingType::Uniform,
					has_dynamic_offset: false,
					min_binding_size: None,
				},
				count: None,
			},
			camera_data.buffer.as_entire_binding(),
		),
		(
			// shadow_caster: proj_mat
			wgpu::BindGroupLayoutEntry {
				binding: 0, // this is automatically updated
				visibility: wgpu::ShaderStages::FRAGMENT,
				ty: wgpu::BindingType::Buffer {
					ty: wgpu::BufferBindingType::Uniform,
					has_dynamic_offset: false,
					min_binding_size: None,
				},
				count: None,
			},
			shadow_caster_data.proj_mat_buffer.as_entire_binding(),
		),
		(
			// material: sampler
			wgpu::BindGroupLayoutEntry {
				binding: 0,
				visibility: wgpu::ShaderStages::FRAGMENT,
				ty: wgpu::BindingType::Sampler (wgpu::SamplerBindingType::Filtering),
				count: None,
			},
			wgpu::BindingResource::Sampler (&sampler),
		),
		(
			// shadowmap: texture
			wgpu::BindGroupLayoutEntry {
				binding: 0,
				visibility: wgpu::ShaderStages::FRAGMENT,
				ty: wgpu::BindingType::Texture {
					multisampled: false,
					view_dimension: wgpu::TextureViewDimension::D2,
					sample_type: wgpu::TextureSampleType::Depth,
				},
				count: None,
			},
			wgpu::BindingResource::TextureView (&shadow_caster_data.depth_tex_view),
		),
		(
			// shadowmap: sampler
			wgpu::BindGroupLayoutEntry {
				binding: 0,
				visibility: wgpu::ShaderStages::FRAGMENT,
				ty: wgpu::BindingType::Sampler (wgpu::SamplerBindingType::Comparison),
				count: None,
			},
			wgpu::BindingResource::Sampler (&depth_tex_sampler),
		),
		(
			// shadowmap: debug_sampler
			wgpu::BindGroupLayoutEntry {
				binding: 0,
				visibility: wgpu::ShaderStages::FRAGMENT,
				ty: wgpu::BindingType::Sampler (wgpu::SamplerBindingType::Filtering),
				count: None,
			},
			wgpu::BindingResource::Sampler (&debug_depth_tex_sampler),
		),
	));
	
	let render_pipeline_layout = render_context.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
		label: Some("Models Render Pipeline"),
		bind_group_layouts: &[
			&bind_group_0_layout,
			&binding_1_layout,
		],
		push_constant_ranges: &[],
	});
	
	let pipeline = render_context.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
		label: Some("Models Render Pipeline"),
		layout: Some(&render_pipeline_layout),
		vertex: wgpu::VertexState {
			module: &shader,
			entry_point: "vs_main",
			buffers: &[
				BasicVertexData::get_layout(),
				ExtendedVertexData::get_layout(),
				RawInstanceData::get_layout(),
			],
			compilation_options: wgpu::PipelineCompilationOptions::default(),
		},
		fragment: Some(wgpu::FragmentState {
			module: &shader,
			entry_point: "fs_main",
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
			depth_write_enabled: true,
			depth_compare: wgpu::CompareFunction::Less,
			stencil: wgpu::StencilState::default(),
			bias: wgpu::DepthBiasState::default(),
		}),
		multisample: wgpu::MultisampleState {
			count: 1,
			mask: !0u64,
			alpha_to_coverage_enabled: false,
		},
		multiview: None,
	});
	
	Ok((
		pipeline,
		bind_group_0_layout,
		bind_group_0,
		//bind_group_1_layout,
	))
}





pub fn load_skybox_render_pipeline(
	render_context: &RenderContextData,
	camera_data: &CameraRenderData,
	skybox_view: &wgpu::TextureView,
) -> Result<(
	wgpu::RenderPipeline,
	wgpu::BindGroupLayout,
	wgpu::BindGroup,
)> {
	
	let shader_path = utils::get_program_file_path("shaders/skybox.wgsl");
	let shader_source = fs::read_to_string(&shader_path).add_path_to_error(&shader_path)?;
	let shader = render_context.device.create_shader_module(wgpu::ShaderModuleDescriptor {
		label: Some("Skybox Render Pipeline"),
		source: wgpu::ShaderSource::Wgsl(shader_source.into()),
	});
	
	let sampler = render_context.device.create_sampler(&wgpu::SamplerDescriptor {
		address_mode_u: wgpu::AddressMode::ClampToEdge,
		address_mode_v: wgpu::AddressMode::ClampToEdge,
		address_mode_w: wgpu::AddressMode::ClampToEdge,
		mag_filter: wgpu::FilterMode::Linear,
		min_filter: wgpu::FilterMode::Nearest,
		mipmap_filter: wgpu::FilterMode::Nearest,
		..Default::default()
	});
	let (bind_group_0_layout, bind_group_0) = get_full_bind_group_data(Some("skybox_bind_group_0"), &render_context.device, vec!(
		(
			// camera: proj_view_mat, inv_proj_mat, view_mat
			wgpu::BindGroupLayoutEntry {
				binding: 0,
				visibility: wgpu::ShaderStages::VERTEX,
				ty: wgpu::BindingType::Buffer {
					ty: wgpu::BufferBindingType::Uniform,
					has_dynamic_offset: false,
					min_binding_size: None,
				},
				count: None,
			},
			camera_data.buffer.as_entire_binding(),
		),
		(
			// skybox: texture
			wgpu::BindGroupLayoutEntry {
				binding: 0,
				visibility: wgpu::ShaderStages::FRAGMENT,
				ty: wgpu::BindingType::Texture {
					multisampled: false,
					view_dimension: wgpu::TextureViewDimension::Cube,
					sample_type: wgpu::TextureSampleType::Float { filterable: true },
				},
				count: None,
			},
			wgpu::BindingResource::TextureView (&skybox_view),
		),
		(
			// skybox: sampler
			wgpu::BindGroupLayoutEntry {
				binding: 0,
				visibility: wgpu::ShaderStages::FRAGMENT,
				ty: wgpu::BindingType::Sampler (wgpu::SamplerBindingType::Filtering),
				count: None,
			},
			wgpu::BindingResource::Sampler (&sampler),
		),
	));
	
	let render_pipeline_layout = render_context.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
		label: Some("Skybox Render Pipeline"),
		bind_group_layouts: &[
			&bind_group_0_layout
		],
		push_constant_ranges: &[],
	});
	
	let pipeline = render_context.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
		label: Some("Skybox Render Pipeline"),
		layout: Some(&render_pipeline_layout),
		vertex: wgpu::VertexState {
			module: &shader,
			entry_point: "vs_main",
			buffers: &[],
			compilation_options: wgpu::PipelineCompilationOptions::default(),
		},
		fragment: Some(wgpu::FragmentState {
			module: &shader,
			entry_point: "fs_main",
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
			front_face: wgpu::FrontFace::Cw,
			cull_mode: Some(wgpu::Face::Back), // todo: change this to None and draw 2 smaller triangles? easiest way is to generate coords (-1, -1), (-1, 1), (1, -1), (1, 1), but that gives one ccw and one cw. There's probably not anything wrong with one large tri though
			polygon_mode: wgpu::PolygonMode::Fill,
			unclipped_depth: false,
			conservative: false,
		},
		depth_stencil: Some(wgpu::DepthStencilState {
			format: wgpu::TextureFormat::Depth32Float,
			depth_write_enabled: true,
			depth_compare: wgpu::CompareFunction::LessEqual,
			stencil: wgpu::StencilState::default(),
			bias: wgpu::DepthBiasState::default(),
		}),
		multisample: wgpu::MultisampleState {
			count: 1,
			mask: !0u64,
			alpha_to_coverage_enabled: false,
		},
		multiview: None,
	});
	
	Ok((
		pipeline,
		bind_group_0_layout,
		bind_group_0,
	))
}





pub fn get_full_bind_group_data(label: Option<&str>, device: &wgpu::Device, entries: Vec<(wgpu::BindGroupLayoutEntry, wgpu::BindingResource)>) -> (wgpu::BindGroupLayout, wgpu::BindGroup) {
	
	// unzip & set binding index
	let (bind_group_layout_entries, bind_group_entries) =
		entries.into_iter().enumerate()
		.fold((vec!(), vec!()), |(mut bind_layout_acc, mut bind_acc), (i, (mut bind_layout_entry, mut bind_entry))| {
			bind_layout_entry.binding = i as u32;
			bind_layout_acc.push(bind_layout_entry);
			bind_acc.push(wgpu::BindGroupEntry {
				binding: i as u32,
				resource: bind_entry,
			});
			(bind_layout_acc, bind_acc)
		});
	
	let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
		label,
		entries: &bind_group_layout_entries,
	});
	let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
		layout: &bind_group_layout,
		entries: &bind_group_entries,
		label,
	});
	
	(bind_group_layout, bind_group)
}



pub fn get_bind_group_layout_data(label: Option<&str>, device: &wgpu::Device, entries: Vec<wgpu::BindGroupLayoutEntry>) -> wgpu::BindGroupLayout {
	
	// set binding index
	let bind_group_layout_entries =
		entries.into_iter().enumerate()
		.fold(vec!(), |mut bind_layout_acc, (i, mut bind_layout_entry)| {
			bind_layout_entry.binding = i as u32;
			bind_layout_acc.push(bind_layout_entry);
			bind_layout_acc
		});
	
	let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
		label,
		entries: &bind_group_layout_entries,
	});
	
	bind_group_layout
}

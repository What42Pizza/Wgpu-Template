use crate::prelude::*;



pub fn load_render_pipelines(render_context: &RenderContextData, generic_bind_layouts: &GenericBindLayouts, render_assets: &RenderAssets) -> Result<RenderPipelines> {
	
	let shadowmap = load_shadowmap_render_pipeline(&render_assets.camera.bind_layout, render_context).context("Failed to load shadowmap render pipeline.")?;
	let example_model = load_example_model_render_pipeline(&render_assets.camera.bind_layout, generic_bind_layouts, render_context).context("Failed to load example model render pipeline.")?;
	let skybox = load_skybox_render_pipeline(&render_assets.camera.bind_layout, generic_bind_layouts, render_context).context("Failed to load skybox render pipeline.")?;
	
	Ok(RenderPipelines {
		shadowmap,
		example_model,
		skybox,
	})
}





pub fn load_shadowmap_render_pipeline(
	proj_bind_layout: &wgpu::BindGroupLayout,
	render_context: &RenderContextData,
) -> Result<wgpu::RenderPipeline> {
	
	let shader_path = utils::get_program_file_path("shaders/shadowmap.wgsl");
	let shader_source = fs::read_to_string(&shader_path).add_path_to_error(&shader_path)?;
	let shader = render_context.device.create_shader_module(wgpu::ShaderModuleDescriptor {
		label: Some("Shadowmap Render Pipeline"),
		source: wgpu::ShaderSource::Wgsl(shader_source.into()),
	});
	
	let render_pipeline_layout = render_context.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
		label: Some("Shadowmap Render Pipeline"),
		bind_group_layouts: &[
			proj_bind_layout,
		],
		push_constant_ranges: &[],
	});
	
	let render_pipeline = render_context.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
		label: Some("Shadowmap Render Pipeline"),
		layout: Some(&render_pipeline_layout),
		vertex: wgpu::VertexState {
			module: &shader,
			entry_point: "vs_main",
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
	
	Ok(render_pipeline)
}





pub fn load_example_model_render_pipeline(
	camera_bind_layout: &wgpu::BindGroupLayout,
	generic_bind_layouts: &GenericBindLayouts,
	render_context: &RenderContextData,
) -> Result<wgpu::RenderPipeline> {
	
	let shader_path = utils::get_program_file_path("shaders/example.wgsl");
	let shader_source = fs::read_to_string(&shader_path).add_path_to_error(&shader_path)?;
	let shader = render_context.device.create_shader_module(wgpu::ShaderModuleDescriptor {
		label: Some("Example Render Pipeline"),
		source: wgpu::ShaderSource::Wgsl(shader_source.into()),
	});
	
	let render_pipeline_layout = render_context.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
		label: Some("Example Render Pipeline"),
		bind_group_layouts: &[
			camera_bind_layout,
			&generic_bind_layouts.texture_2d,
		],
		push_constant_ranges: &[],
	});
	
	let render_pipeline = render_context.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
		label: Some("Example Render Pipeline"),
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
	
	Ok(render_pipeline)
}





pub fn load_skybox_render_pipeline(
	camera_bind_layout: &wgpu::BindGroupLayout,
	generic_bind_layouts: &GenericBindLayouts,
	render_context: &RenderContextData,
) -> Result<wgpu::RenderPipeline> {
	
	let shader_path = utils::get_program_file_path("shaders/skybox.wgsl");
	let shader_source = fs::read_to_string(&shader_path).add_path_to_error(&shader_path)?;
	let shader = render_context.device.create_shader_module(wgpu::ShaderModuleDescriptor {
		label: Some("Skybox Render Pipeline"),
		source: wgpu::ShaderSource::Wgsl(shader_source.into()),
	});
	
	let render_pipeline_layout = render_context.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
		label: Some("Skybox Render Pipeline"),
		bind_group_layouts: &[
			camera_bind_layout,
			&generic_bind_layouts.texture_cube,
		],
		push_constant_ranges: &[],
	});
	
	let render_pipeline = render_context.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
	
	Ok(render_pipeline)
}

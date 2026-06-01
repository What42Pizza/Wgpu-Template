use crate::*;



pub struct PostProcessingLayouts {
	pub render_pipeline: wgpu::RenderPipeline,
	pub bind_group_0: wgpu::BindGroupLayout,
}

pub struct PostProcessingBindings (wgpu::BindGroup);



pub fn load_layout(render_context: &RenderContext) -> Result<PostProcessingLayouts> {
	
	let shader_path = utils::get_program_file_path("shaders/post_processing.wgsl");
	let shader_source = fs_read_to_string(&shader_path)?;
	let shader = render_context.gpu_device.create_shader_module(wgpu::ShaderModuleDescriptor {
		label: Some("post_processing_shader_module"),
		source: wgpu::ShaderSource::Wgsl(shader_source.into()),
	});
	
	let bind_group_0 = render_context.gpu_device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
		label: Some("post_processing_bind_0_layout"),
		entries: &[
			wgpu::BindGroupLayoutEntry { // post_processing: buffer
				binding: 0,
				visibility: wgpu::ShaderStages::FRAGMENT,
				ty: wgpu::BindingType::Buffer {
					ty: wgpu::BufferBindingType::Uniform,
					has_dynamic_offset: false,
					min_binding_size: None,
				},
				count: None,
			},
			wgpu::BindGroupLayoutEntry { // main_tex: texture
				binding: 1,
				visibility: wgpu::ShaderStages::FRAGMENT,
				ty: wgpu::BindingType::Texture {
					multisampled: false,
					view_dimension: wgpu::TextureViewDimension::D2,
					sample_type: wgpu::TextureSampleType::Float { filterable: true },
				},
				count: None,
			},
			wgpu::BindGroupLayoutEntry { // main_tex: sampler
				binding: 2,
				visibility: wgpu::ShaderStages::FRAGMENT,
				ty: wgpu::BindingType::Sampler (wgpu::SamplerBindingType::Filtering),
				count: None,
			},
		],
	});
	
	let pipeline_layout = render_context.gpu_device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
		label: Some("post_processing_pipeline_layout"),
		bind_group_layouts: &[
			Some(&bind_group_0),
		],
		immediate_size: 0,
	});
	let render_pipeline = render_context.gpu_device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
		label: Some("post_processing_pipeline"),
		layout: Some(&pipeline_layout),
		vertex: wgpu::VertexState {
			module: &shader,
			entry_point: Some("vs_main"),
			buffers: &[],
			compilation_options: wgpu::PipelineCompilationOptions::default(),
		},
		fragment: Some(wgpu::FragmentState {
			module: &shader,
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
			front_face: wgpu::FrontFace::Cw,
			cull_mode: Some(wgpu::Face::Back),
			polygon_mode: wgpu::PolygonMode::Fill,
			unclipped_depth: false,
			conservative: false,
		},
		depth_stencil: None,
		multisample: wgpu::MultisampleState {
			count: 1,
			mask: u64::MAX,
			alpha_to_coverage_enabled: false,
		},
		multiview_mask: None,
		cache: None,
	});
	
	Ok(PostProcessingLayouts {
		render_pipeline,
		bind_group_0,
	})
}



pub fn load_bindings(context: &RenderContext, layouts: &PostProcessingLayouts, assets: &RenderAssets) -> PostProcessingBindings {
	
	let bind_group_0 = context.gpu_device.create_bind_group(&wgpu::BindGroupDescriptor {
		label: Some("post_processing_bind_0"),
		layout: &layouts.bind_group_0,
		entries: &[
			wgpu::BindGroupEntry {
				binding: 0,
				resource: assets.post_processing_buffer.as_entire_binding(),
			},
			wgpu::BindGroupEntry {
				binding: 1,
				resource: wgpu::BindingResource::TextureView (&assets.main_tex_view),
			},
			wgpu::BindGroupEntry {
				binding: 2,
				resource: wgpu::BindingResource::Sampler (&assets.default_sampler),
			},
		],
	});
	
	PostProcessingBindings (bind_group_0)
}



pub fn render(layouts: &PostProcessingLayouts, assets: &RenderAssets, bindings: &PostProcessingBindings, encoder: &mut wgpu::CommandEncoder, output_view: &wgpu::TextureView) {
	
	let mut skybox_pass_handle = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
		label: Some("color_correction_render_pass"),
		color_attachments: &[Some(wgpu::RenderPassColorAttachment {
			view: output_view,
			depth_slice: None,
			resolve_target: None,
			ops: wgpu::Operations {
				load: wgpu::LoadOp::Load,
				store: wgpu::StoreOp::Store,
			},
		})],
		depth_stencil_attachment: None,
		multiview_mask: None,
		occlusion_query_set: None,
		timestamp_writes: None,
	});
	
	skybox_pass_handle.set_pipeline(&layouts.render_pipeline);
	skybox_pass_handle.set_bind_group(0, &bindings.0, &[]);
	
	skybox_pass_handle.draw(0..3, 0..1)
	
}

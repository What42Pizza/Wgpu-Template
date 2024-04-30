use crate::prelude::*;
use async_std::task::block_on;
use winit::window::Window;





pub struct RenderContextData<'a> {
	pub drawable_surface: wgpu::Surface<'a>,
	pub device: wgpu::Device,
	pub command_queue: wgpu::Queue,
	pub surface_config: wgpu::SurfaceConfiguration,
	pub size: winit::dpi::PhysicalSize<u32>,
	pub aspect_ratio: f32,
}



pub struct TextureBindData {
	pub group: wgpu::BindGroup,
	pub layout: wgpu::BindGroupLayout,
}

pub struct DepthTextureData {
	pub texture: wgpu::Texture,
	pub view: wgpu::TextureView,
	pub sampler: wgpu::Sampler,
}

pub struct GeneralBindData {
	pub buffer: wgpu::Buffer,
	pub group: wgpu::BindGroup,
	pub layout: wgpu::BindGroupLayout,
}





pub fn init_wgpu_context_data<'a>(window: &'a Window, engine_config: &load::EngineConfig) -> Result<RenderContextData<'a>> {
	block_on(init_wgpu_context_data_async(window, engine_config))
}

pub async fn init_wgpu_context_data_async<'a>(window: &'a Window, engine_config: &load::EngineConfig) -> Result<RenderContextData<'a>> {
	let size = window.inner_size();
	
	// The instance is a handle to our GPU
	// Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
	let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
		backends: engine_config.rendering_backend,
		..Default::default()
	});
	
	// Handle to a presentable surface
	let surface = instance.create_surface(window)?;
	
	// Handle to a physical graphics and/or compute device
	let mut adapter = instance.request_adapter(
		&wgpu::RequestAdapterOptions {
			power_preference: wgpu::PowerPreference::default(),
			compatible_surface: Some(&surface),
			force_fallback_adapter: false,
		},
	).await;
	if adapter.is_none() {
		adapter =
			instance
			.enumerate_adapters(wgpu::Backends::all()).into_iter()
			.find(|adapter| adapter.is_surface_supported(&surface));
	}
	let Some(adapter) = adapter else {return Err(Error::msg("Unable to find suitable adapter."));};
	
	// Open connection to a graphics and/or compute device, Handle to a command queue on a device
	let (device, queue) = adapter.request_device(
		&wgpu::DeviceDescriptor {
			required_features: wgpu::Features::empty(),
			required_limits: wgpu::Limits::downlevel_defaults(),
			label: None,
		},
		None,
	).await?;
	
	let surface_caps = surface.get_capabilities(&adapter);
	let surface_format = surface_caps.formats.iter()
		.copied()
		.find(|f| f.is_srgb())
		.unwrap_or(surface_caps.formats[0]);
	let config = wgpu::SurfaceConfiguration {
		usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
		format: surface_format,
		width: size.width,
		height: size.height,
		present_mode: engine_config.present_mode,
		alpha_mode: surface_caps.alpha_modes[0],
		view_formats: vec![],
		desired_maximum_frame_latency: engine_config.desired_frame_latency,
	};
	surface.configure(&device, &config);
	
	Ok(RenderContextData {
		drawable_surface: surface,
		device,
		command_queue: queue,
		surface_config: config,
		size,
		aspect_ratio: size.width as f32 / size.height as f32,
	})
}





pub fn init_wgpu_pipeline(
	name: &str,
	shader_path: impl AsRef<Path>,
	bind_group_layouts: &[&wgpu::BindGroupLayout],
	buffer_layouts: &[wgpu::VertexBufferLayout],
	render_context: &RenderContextData,
) -> Result<wgpu::RenderPipeline> {
	
	let shader_source = fs::read_to_string(shader_path)?;
	let shader = render_context.device.create_shader_module(wgpu::ShaderModuleDescriptor {
		label: Some(name),
		source: wgpu::ShaderSource::Wgsl(shader_source.into()),
	});
	
	let render_pipeline_layout = render_context.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
		label: Some(name),
		bind_group_layouts,
		push_constant_ranges: &[],
	});
	
	let render_pipeline = render_context.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
		label: Some(name),
		layout: Some(&render_pipeline_layout),
		vertex: wgpu::VertexState {
			module: &shader,
			entry_point: "vs_main",
			buffers: buffer_layouts,
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





pub fn load_texture(path: impl AsRef<Path>, render_context: &RenderContextData) -> Result<TextureBindData> {
	
	let raw_texture_bytes = fs::read(utils::get_program_file_path(path))?;
	let texture_bytes = image::load_from_memory(&raw_texture_bytes)?;
	let texture_bytes = texture_bytes.to_rgba8();
	let dimensions = texture_bytes.dimensions();
	
	let texture_size = wgpu::Extent3d {
		width: dimensions.0,
		height: dimensions.1,
		depth_or_array_layers: 1,
	};
	let texture = render_context.device.create_texture(
		&wgpu::TextureDescriptor {
			size: texture_size,
			mip_level_count: 1,
			sample_count: 1,
			dimension: wgpu::TextureDimension::D2,
			format: wgpu::TextureFormat::Rgba8UnormSrgb,
			// TEXTURE_BINDING tells wgpu that we want to use this texture in shaders
			// COPY_DST means that we want to copy data to this texture
			usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
			label: Some("Diffuse Texture"),
			// This is the same as with the SurfaceConfig. It
			// specifies what texture formats can be used to
			// create TextureViews for this texture. The base
			// texture format (Rgba8UnormSrgb in this case) is
			// always supported. Note that using a different
			// texture format is not supported on the WebGL2
			// backend.
			view_formats: &[],
		}
	);
	
	render_context.command_queue.write_texture(
		wgpu::ImageCopyTexture {
			texture: &texture,
			mip_level: 0,
			origin: wgpu::Origin3d::ZERO,
			aspect: wgpu::TextureAspect::All,
		},
		&texture_bytes,
		wgpu::ImageDataLayout {
			offset: 0,
			bytes_per_row: Some(4 * dimensions.0),
			rows_per_image: Some(dimensions.1),
		},
		texture_size,
	);
	
	let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
	let sampler = render_context.device.create_sampler(&wgpu::SamplerDescriptor {
		address_mode_u: wgpu::AddressMode::ClampToEdge,
		address_mode_v: wgpu::AddressMode::ClampToEdge,
		address_mode_w: wgpu::AddressMode::ClampToEdge,
		mag_filter: wgpu::FilterMode::Linear,
		min_filter: wgpu::FilterMode::Nearest,
		mipmap_filter: wgpu::FilterMode::Nearest,
		..Default::default()
	});
	
	let bind_group_layout = render_context.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
		entries: &[
			wgpu::BindGroupLayoutEntry {
				binding: 0,
				visibility: wgpu::ShaderStages::FRAGMENT,
				ty: wgpu::BindingType::Texture {
					multisampled: false,
					view_dimension: wgpu::TextureViewDimension::D2,
					sample_type: wgpu::TextureSampleType::Float { filterable: true },
				},
				count: None,
			},
			wgpu::BindGroupLayoutEntry {
				binding: 1,
				visibility: wgpu::ShaderStages::FRAGMENT,
				ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
				count: None,
			},
		],
		label: Some("texture_bind_group_layout"),
	});
	
	let bind_group = render_context.device.create_bind_group(
		&wgpu::BindGroupDescriptor {
			layout: &bind_group_layout,
			entries: &[
				wgpu::BindGroupEntry {
					binding: 0,
					resource: wgpu::BindingResource::TextureView(&texture_view),
				},
				wgpu::BindGroupEntry {
					binding: 1,
					resource: wgpu::BindingResource::Sampler(&sampler),
				}
			],
			label: Some("diffuse_bind_group"),
		}
	);
	
	Ok(TextureBindData {
		group: bind_group,
		layout: bind_group_layout,
	})
}



pub fn create_depth_texture(label: &str, render_context: &RenderContextData) -> DepthTextureData {
	
	let size = wgpu::Extent3d {
		width: render_context.surface_config.width,
		height: render_context.surface_config.height,
		depth_or_array_layers: 1,
	};
	let desc = wgpu::TextureDescriptor {
		label: Some(label),
		size,
		mip_level_count: 1,
		sample_count: 1,
		dimension: wgpu::TextureDimension::D2,
		format: wgpu::TextureFormat::Depth32Float,
		usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
		view_formats: &[],
	};
	let texture = render_context.device.create_texture(&desc);
	
	let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
	let sampler = render_context.device.create_sampler(
		&wgpu::SamplerDescriptor {
			address_mode_u: wgpu::AddressMode::ClampToEdge,
			address_mode_v: wgpu::AddressMode::ClampToEdge,
			address_mode_w: wgpu::AddressMode::ClampToEdge,
			mag_filter: wgpu::FilterMode::Linear,
			min_filter: wgpu::FilterMode::Linear,
			mipmap_filter: wgpu::FilterMode::Nearest,
			compare: Some(wgpu::CompareFunction::LessEqual),
			lod_min_clamp: 0.0,
			lod_max_clamp: 100.0,
			..Default::default()
		}
	);
	
	DepthTextureData {
		texture,
		view,
		sampler
	}
}

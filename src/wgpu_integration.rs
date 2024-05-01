use crate::prelude::*;
use std::io::{BufReader, Cursor};
use async_std::task::block_on;
use wgpu::util::DeviceExt;
use winit::window::Window;





pub struct RenderContextData<'a> {
	pub drawable_surface: wgpu::Surface<'a>,
	pub device: wgpu::Device,
	pub command_queue: wgpu::Queue,
	pub surface_config: wgpu::SurfaceConfiguration,
	pub size: winit::dpi::PhysicalSize<u32>,
	pub aspect_ratio: f32,
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





pub fn load_texture(path: impl AsRef<Path>, render_context: &RenderContextData) -> Result<TextureData> {
	
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
			usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
			label: Some("Diffuse Texture"),
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
	
	let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
	let sampler = render_context.device.create_sampler(&wgpu::SamplerDescriptor {
		address_mode_u: wgpu::AddressMode::ClampToEdge,
		address_mode_v: wgpu::AddressMode::ClampToEdge,
		address_mode_w: wgpu::AddressMode::ClampToEdge,
		mag_filter: wgpu::FilterMode::Linear,
		min_filter: wgpu::FilterMode::Nearest,
		mipmap_filter: wgpu::FilterMode::Nearest,
		..Default::default()
	});
	
	Ok(TextureData {
		texture,
		view,
		sampler,
	})
}





pub fn load_model(
	file_path: impl AsRef<Path>,
	render_context: &RenderContextData,
	layout: &wgpu::BindGroupLayout,
) -> Result<(Vec<MeshRenderData>, Vec<MaterialRenderData>)> {
	let file_path = file_path.as_ref();
	let obj_text = fs::read_to_string(file_path)?;
	let obj_cursor = Cursor::new(obj_text);
	let mut obj_reader = BufReader::new(obj_cursor);
	let parent_path = file_path.parent().expect("Cannot load mesh at root directory");
	
	let (models, obj_materials) = tobj::load_obj_buf(
		&mut obj_reader,
		&tobj::LoadOptions {
			triangulate: true,
			single_index: true,
			..Default::default()
		},
		move |p| {
			let mat_text = fs::read_to_string(parent_path.join(p)).map_err(|_err| tobj::LoadError::OpenFileFailed)?;
			tobj::load_mtl_buf(&mut BufReader::new(Cursor::new(mat_text)))
		}
	)?;
	let obj_materials = obj_materials?;
	
	let mut materials = Vec::new();
	for material in obj_materials {
		let Some(diffuse_texture_path) = material.diffuse_texture.as_ref() else {
			warn!("diffuse texture in material is `None`.");
			continue;
		};
		let texture = load_texture(parent_path.join(diffuse_texture_path), render_context)?;
		let bind_group = render_context.device.create_bind_group(&wgpu::BindGroupDescriptor {
			layout,
			entries: &[
				wgpu::BindGroupEntry {
					binding: 0,
					resource: wgpu::BindingResource::TextureView(&texture.view),
				},
				wgpu::BindGroupEntry {
					binding: 1,
					resource: wgpu::BindingResource::Sampler(&texture.sampler),
				},
			],
			label: None,
		});
		
		materials.push(MaterialRenderData {
			texture: texture.texture,
			view: texture.view,
			sampler: texture.sampler,
			bind_group,
		})
	}
	
	let meshes = models
		.into_iter()
		.map(|model| {
			let vertices = (0..model.mesh.positions.len() / 3)
				.map(|i| GenericVertex {
					position: [
						model.mesh.positions[i * 3],
						model.mesh.positions[i * 3 + 1],
						model.mesh.positions[i * 3 + 2],
					],
					tex_coords: [model.mesh.texcoords[i * 2], 1.0 - model.mesh.texcoords[i * 2 + 1]],
					normal: [
						model.mesh.normals[i * 3],
						model.mesh.normals[i * 3 + 1],
						model.mesh.normals[i * 3 + 2],
					],
				})
				.collect::<Vec<_>>();
			
			let vertex_buffer = render_context.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
				label: Some(&format!("{:?} Vertex Buffer", &file_path)),
				contents: bytemuck::cast_slice(&vertices),
				usage: wgpu::BufferUsages::VERTEX,
			});
			let index_buffer = render_context.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
				label: Some(&format!("{:?} Index Buffer", &file_path)),
				contents: bytemuck::cast_slice(&model.mesh.indices),
				usage: wgpu::BufferUsages::INDEX,
			});
			
			MeshRenderData {
				vertex_buffer,
				index_buffer,
				index_count: model.mesh.indices.len() as u32,
				material_index: model.mesh.material_id.unwrap_or(0),
			}
		})
		.collect::<Vec<_>>();
	
	Ok((
		meshes,
		materials,
	))
}

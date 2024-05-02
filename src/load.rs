use crate::prelude::*;
use std::io::{BufReader, Cursor};
use serde_hjson::{Map, Value};
use wgpu::{util::DeviceExt, BindGroupLayout};
use cgmath::prelude::*;



pub fn load_program_data(start_time: Instant, window: &Window) -> Result<ProgramData> {
	
	let engine_config = load_engine_config()?;
	let camera = Camera::new((0., 1., 2.));
	
	let render_context = wgpu_integration::load_render_context_data(window, &engine_config)?;
	let render_layouts = load_texture_layouts(&render_context);
	let render_assets = load_render_assets(&render_context, &render_layouts)?;
	let render_pipelines = load_render_pipelines(&render_context, &render_layouts, &render_assets)?;
	
	Ok(ProgramData {
		
		start_time,
		window,
		pressed_keys: HashMap::new(),
		frame_start_instant: start_time,
		min_frame_time: engine_config.min_frame_time,
		fps_counter: FpsCounter::new(),
		
		render_context,
		render_layouts,
		render_assets,
		render_pipelines,
		
		camera,
		
	})
}



pub struct EngineConfig {
	pub rendering_backend: wgpu::Backends,
	pub present_mode: wgpu::PresentMode,
	pub desired_frame_latency: u32,
	pub min_frame_time: Duration,
}

pub fn load_engine_config() -> Result<EngineConfig> {
	
	let engine_config_path = utils::get_program_file_path("engine config.hjson");
	let engine_config_result = fs::read_to_string(engine_config_path);
	let engine_config_string = match &engine_config_result {
		StdResult::Ok(v) => &**v,
		StdResult::Err(err) => {
			warn!("Failed to read 'engine config.hjson', using default values...  (error: {err})");
			include_str!("../data/default engine config.hjson")
		}
	};
	let engine_config: Map<String, Value> = serde_hjson::from_str(engine_config_string).context("Failed to decode 'engine config.hjson'")?;
	
	let rendering_backend_str = read_hjson_str(&engine_config, "rendering_backend", "auto");
	let rendering_backend = match &*rendering_backend_str.to_lowercase() {
		"auto" => wgpu::Backends::all(),
		"vulkan" => wgpu::Backends::VULKAN,
		"dx12" => wgpu::Backends::DX12,
		"metal" => wgpu::Backends::METAL,
		"opengl" => wgpu::Backends::GL,
		_ => {
			warn!("Unknown value for entry 'rendering_backend' in 'engine config.hjson', must be: 'auto', 'vulkan', 'dx12', 'metal', or 'opengl', defaulting to \"auto\".");
			wgpu::Backends::all()
		}
	};
	
	let present_mode_str = read_hjson_str(&engine_config, "present_mode", "auto_vsync");
	let present_mode = match &*present_mode_str.to_lowercase() {
		"auto_vsync" => wgpu::PresentMode::AutoVsync,
		"auto_no_vsync" => wgpu::PresentMode::AutoVsync,
		"fifo" => wgpu::PresentMode::Fifo,
		"fifo_relaxed" => wgpu::PresentMode::FifoRelaxed,
		"immediate" => wgpu::PresentMode::Immediate,
		"mailbox" => wgpu::PresentMode::Mailbox,
		_ => {
			warn!("Unknown value for entry 'present_mode' in 'engine config.hjson', must be: 'auto_vsync', 'auto_no_vsync', 'fifo', 'fifo_relaxed', 'immediate', or 'mailbox', defaulting to \"auto_vsync\".");
			wgpu::PresentMode::AutoVsync
		}
	};
	
	let desired_frame_latency_i64 = read_hjson_i64(&engine_config, "desired_frame_latency", 1);
	let desired_frame_latency = desired_frame_latency_i64 as u32;
	
	let min_frame_time_f64 = read_hjson_f64(&engine_config, "min_frame_time", 0.002);
	let min_frame_time = Duration::from_secs_f64(min_frame_time_f64);
	
	Ok(EngineConfig {
		rendering_backend,
		present_mode,
		desired_frame_latency,
		min_frame_time,
	})
}

pub fn read_hjson_str<'a>(map: &'a Map<String, Value>, key: &'static str, default: &'static str) -> &'a str {
	let value_str = map.get(key);
	let value_str = value_str.map(|v| v.as_str().unwrap_or_else(|| {
		warn!("Entry '{key}' in 'engine config.hjson' must be a string, defaulting to \"{default}\".");
		default
	}));
	value_str.unwrap_or_else(|| {
		warn!("Could not find entry '{key}' in 'engine config.hjson', defaulting to \"{default}\".");
		default
	})
}

pub fn read_hjson_i64(map: &Map<String, Value>, key: &'static str, default: i64) -> i64 {
	let value_str = map.get(key);
	let value_i64 = value_str.map(|v| v.as_i64().unwrap_or_else(|| {
		warn!("Entry '{key}' in 'engine config.hjson' must be an int, defaulting to \"{default}\".");
		default
	}));
	value_i64.unwrap_or_else(|| {
		warn!("Could not find entry '{key}' in 'engine config.hjson', defaulting to \"{default}\".");
		default
	})
}

pub fn read_hjson_f64(map: &Map<String, Value>, key: &'static str, default: f64) -> f64 {
	let value_str = map.get(key);
	let value_f64 = value_str.map(|v| v.as_f64().unwrap_or_else(|| {
		warn!("Entry '{key}' in 'engine config.hjson' must be an int, defaulting to \"{default}\".");
		default
	}));
	value_f64.unwrap_or_else(|| {
		warn!("Could not find entry '{key}' in 'engine config.hjson', defaulting to \"{default}\".");
		default
	})
}





pub fn load_texture_layouts(render_context: &wgpu_integration::RenderContextData) -> TextureLayouts {
	
	let generic = render_context.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
		entries: &[
			wgpu::BindGroupLayoutEntry { // texture view
				binding: 0,
				visibility: wgpu::ShaderStages::FRAGMENT,
				ty: wgpu::BindingType::Texture {
					multisampled: false,
					view_dimension: wgpu::TextureViewDimension::D2,
					sample_type: wgpu::TextureSampleType::Float { filterable: true },
				},
				count: None,
			},
			wgpu::BindGroupLayoutEntry { // sampler
				binding: 1,
				visibility: wgpu::ShaderStages::FRAGMENT,
				ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
				count: None,
			},
		],
		label: Some("texture_bind_group_layout"),
	});
	
	let cube = render_context.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
		entries: &[
			wgpu::BindGroupLayoutEntry { // texture view
				binding: 0,
				visibility: wgpu::ShaderStages::FRAGMENT,
				ty: wgpu::BindingType::Texture {
					multisampled: false,
					view_dimension: wgpu::TextureViewDimension::Cube,
					sample_type: wgpu::TextureSampleType::Float { filterable: true },
				},
				count: None,
			},
			wgpu::BindGroupLayoutEntry { // sampler
				binding: 1,
				visibility: wgpu::ShaderStages::FRAGMENT,
				ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
				count: None,
			},
		],
		label: Some("cube_texture_bind_group_layout"),
	});
	
	TextureLayouts {
		generic,
		cube,
	}
}





pub fn load_render_assets(render_context: &wgpu_integration::RenderContextData, render_layouts: &TextureLayouts) -> Result<RenderAssets> {
	
	let mut materials_storage = MaterialsStorage::new();
	let test_model = load_test_model_render_data(&render_context, &render_layouts.generic, &mut materials_storage)?;
	let depth = load_depth_render_data(&render_context)?;
	let camera = load_camera_render_data(&render_context)?;
	
	Ok(RenderAssets {
		materials_storage,
		test_model,
		depth,
		camera,
	})
}



pub fn load_test_model_render_data(
	render_context: &wgpu_integration::RenderContextData,
	texture_layout: &BindGroupLayout,
	materials_storage: &mut MaterialsStorage,
) -> Result<ModelRenderData> {
	
	let test_model_meshes = load_model(utils::get_program_file_path("assets/cube.obj"), render_context, texture_layout, materials_storage)?;
	
	let test_model_instances = (0..100).flat_map(|z| {
		(0..100).map(move |x| {
			let position = cgmath::Vector3 { x: x as f32 * 3.0, y: 0.0, z: z as f32 * 3.0 } - cgmath::Vector3::new(0.5, 0.0, 0.5);
			
			let rotation = if position.is_zero() {
				cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_z(), cgmath::Deg(0.0))
			} else {
				cgmath::Quaternion::from_axis_angle(position.normalize(), cgmath::Deg(45.0))
			};
			
			Instance {
				position,
				rotation,
			}
		})
	}).collect::<Vec<_>>();
	
	let test_model_instances_data = test_model_instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
	let test_model_instances_buffer = render_context.device.create_buffer_init(
		&wgpu::util::BufferInitDescriptor {
			label: Some("Instance Buffer"),
			contents: bytemuck::cast_slice(&test_model_instances_data),
			usage: wgpu::BufferUsages::VERTEX,
		}
	);
	
	Ok(ModelRenderData {
		instances_buffer: test_model_instances_buffer,
		instances_count: test_model_instances.len() as u32,
		meshes: test_model_meshes,
	})
}



pub fn load_model(
	file_path: impl AsRef<Path>,
	render_context: &wgpu_integration::RenderContextData,
	texture_layout: &wgpu::BindGroupLayout,
	materials_storage: &mut MaterialsStorage,
) -> Result<Vec<MeshRenderData>> {
	let file_path = file_path.as_ref();
	let obj_text = fs::read_to_string(file_path)?;
	let obj_cursor = Cursor::new(obj_text);
	let mut obj_reader = BufReader::new(obj_cursor);
	let parent_folder = file_path.parent().expect("Cannot load mesh at root directory");
	
	let (models, obj_materials) = tobj::load_obj_buf(
		&mut obj_reader,
		&tobj::LoadOptions {
			triangulate: true,
			single_index: true,
			..Default::default()
		},
		move |p| {
			let mat_text = fs::read_to_string(parent_folder.join(p)).map_err(|_err| tobj::LoadError::OpenFileFailed)?;
			tobj::load_mtl_buf(&mut BufReader::new(Cursor::new(mat_text)))
		}
	)?;
	let obj_materials = obj_materials?;
	
	let mut material_ids = Vec::new();
	for material in obj_materials {
		let Some(diffuse_texture_name) = material.diffuse_texture else {
			warn!("diffuse texture in material is `None`.");
			continue;
		};
		let material_id = match load::get_material_index(&diffuse_texture_name, materials_storage) {
			Some(v) => v,
			None => load::load_material(diffuse_texture_name, parent_folder, materials_storage, render_context, texture_layout)?
		};
		material_ids.push(material_id);
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
				material_index: material_ids[model.mesh.material_id.unwrap_or(0)],
			}
		})
		.collect::<Vec<_>>();
	
	Ok(meshes)
}



pub fn get_material_index(name: &str, materials_storage: &MaterialsStorage) -> Option<usize> {
	materials_storage.list.iter().enumerate()
		.find(|(_i, material)| &*material.name == name)
		.map(|(i, _material)| i)
}

pub fn load_material(
	name: String,
	parent_folder: impl AsRef<Path>,
	materials_storage: &mut MaterialsStorage,
	render_context: &wgpu_integration::RenderContextData,
	texture_layout: &BindGroupLayout
) -> Result<usize> {
	let output = materials_storage.list.len();
	let material = wgpu_integration::load_material(name, parent_folder, render_context, texture_layout)?;
	materials_storage.list.push(material);
	Ok(output)
}



pub fn load_depth_render_data(render_context: &wgpu_integration::RenderContextData) -> Result<DepthRenderData> {
	
	let size = wgpu::Extent3d {
		width: render_context.surface_config.width,
		height: render_context.surface_config.height,
		depth_or_array_layers: 1,
	};
	let desc = wgpu::TextureDescriptor {
		label: Some("Depth Texture"),
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
	//let sampler = render_context.device.create_sampler(
	//	&wgpu::SamplerDescriptor {
	//		address_mode_u: wgpu::AddressMode::ClampToEdge,
	//		address_mode_v: wgpu::AddressMode::ClampToEdge,
	//		address_mode_w: wgpu::AddressMode::ClampToEdge,
	//		mag_filter: wgpu::FilterMode::Linear,
	//		min_filter: wgpu::FilterMode::Linear,
	//		mipmap_filter: wgpu::FilterMode::Nearest,
	//		compare: Some(wgpu::CompareFunction::LessEqual),
	//		lod_min_clamp: 0.0,
	//		lod_max_clamp: 100.0,
	//		..Default::default()
	//	}
	//);
	
	Ok(DepthRenderData {
		//texture,
		view,
		//sampler,
	})
}



pub fn load_camera_render_data(render_context: &wgpu_integration::RenderContextData) -> Result<CameraRenderData> {
	
	let bind_layout = render_context.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
		entries: &[
			wgpu::BindGroupLayoutEntry { // proj_mat
				binding: 0,
				visibility: wgpu::ShaderStages::VERTEX,
				ty: wgpu::BindingType::Buffer {
					ty: wgpu::BufferBindingType::Uniform,
					has_dynamic_offset: false,
					min_binding_size: None,
				},
				count: None,
			}
		],
		label: Some("camera_bind_group_layout"),
	});
	
	let initial_data = Camera::default_data();
	let buffer = render_context.device.create_buffer_init(
		&wgpu::util::BufferInitDescriptor {
			label: Some("Camera Buffer"),
			contents: bytemuck::cast_slice(&[initial_data]),
			usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
		}
	);
	
	let bind_group = render_context.device.create_bind_group(&wgpu::BindGroupDescriptor {
		layout: &bind_layout,
		entries: &[
			wgpu::BindGroupEntry { // proj_mat
				binding: 0,
				resource: buffer.as_entire_binding(),
			}
		],
		label: Some("camera_bind_group"),
	});
	
	Ok(CameraRenderData {
		buffer,
		bind_layout,
		bind_group,
	})
}





pub fn load_render_pipelines(render_context: &wgpu_integration::RenderContextData, render_layouts: &TextureLayouts, render_assets: &RenderAssets) -> Result<RenderPipelines> {
	
	let test_render_pipeline = load_test_render_pipeline(&render_assets.camera.bind_layout, &render_layouts.generic, &render_context)?;
	let skybox_render_pipeline = load_skybox_render_pipeline(&render_assets.camera.bind_layout, &render_layouts.cube, &render_context)?; // it's better to have this at the end so that only the necessary pixels are rendered
	
	Ok(RenderPipelines {
		test: test_render_pipeline,
		skybox: skybox_render_pipeline,
	})
}





pub fn load_test_render_pipeline(
	camera_layout: &wgpu::BindGroupLayout,
	texture_layout: &wgpu::BindGroupLayout,
	render_context: &wgpu_integration::RenderContextData,
) -> Result<wgpu::RenderPipeline> {
	
	let shader_source = fs::read_to_string(utils::get_program_file_path("shaders/test_model.wgsl"))?;
	let shader = render_context.device.create_shader_module(wgpu::ShaderModuleDescriptor {
		label: Some("Test Render Pipeline"),
		source: wgpu::ShaderSource::Wgsl(shader_source.into()),
	});
	
	let render_pipeline_layout = render_context.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
		label: Some("Test Render Pipeline"),
		bind_group_layouts: &[
			camera_layout,
			texture_layout,
		],
		push_constant_ranges: &[],
	});
	
	let render_pipeline = render_context.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
		label: Some("Test Render Pipeline"),
		layout: Some(&render_pipeline_layout),
		vertex: wgpu::VertexState {
			module: &shader,
			entry_point: "vs_main",
			buffers: &[
				GenericVertex::get_layout(),
				InstanceRaw::get_layout()
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
	camera_layout: &wgpu::BindGroupLayout,
	cube_texture_layout: &wgpu::BindGroupLayout,
	render_context: &wgpu_integration::RenderContextData,
) -> Result<wgpu::RenderPipeline> {
	
	let shader_source = fs::read_to_string(utils::get_program_file_path("shaders/skybox.wgsl"))?;
	let shader = render_context.device.create_shader_module(wgpu::ShaderModuleDescriptor {
		label: Some("Skybox Render Pipeline"),
		source: wgpu::ShaderSource::Wgsl(shader_source.into()),
	});
	
	let render_pipeline_layout = render_context.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
		label: Some("Skybox Render Pipeline"),
		bind_group_layouts: &[
			camera_layout,
			cube_texture_layout,
		],
		push_constant_ranges: &[],
	});
	
	let render_pipeline = render_context.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
		label: Some("Skybox Render Pipeline"),
		layout: Some(&render_pipeline_layout),
		vertex: wgpu::VertexState {
			module: &shader,
			entry_point: "vs_main",
			buffers: &[
				GenericVertex::get_layout(),
				InstanceRaw::get_layout()
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

use crate::prelude::*;
use serde_hjson::{Map, Value};
use wgpu::util::DeviceExt;
use cgmath::prelude::*;



pub fn init_program_data(start_time: Instant, window: &Window) -> Result<ProgramData> {
	
	let engine_config = load_engine_config()?;
	
	let camera = Camera::new((0., 1., 2.));
	let render_context = wgpu_integration::init_wgpu_context_data(window, &engine_config)?;
	let uniform_datas = load_uniform_datas(&render_context)?;
	let asset_datas = load_asset_datas(&render_context)?;
	let world_datas = load_world_datas(&render_context)?;
	let render_pipelines = load_pipelines(&render_context, &uniform_datas, &asset_datas)?;
	
	Ok(ProgramData {
		
		window,
		pressed_keys: HashMap::new(),
		frame_start_instant: start_time,
		min_frame_time: engine_config.min_frame_time,
		
		render_context,
		render_pipelines,
		uniform_datas,
		asset_datas,
		world_datas,
		camera,
		
		start_time,
		fps_counter: FpsCounter::new(),
		
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
			log!("Unknown value for entry 'rendering_backend' in 'engine config.hjson', must be: 'auto', 'vulkan', 'dx12', 'metal', or 'opengl', defaulting to \"auto\".");
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
			log!("Unknown value for entry 'present_mode' in 'engine config.hjson', must be: 'auto_vsync', 'auto_no_vsync', 'fifo', 'fifo_relaxed', 'immediate', or 'mailbox', defaulting to \"auto_vsync\".");
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
		log!("Entry '{key}' in 'engine config.hjson' must be a string, defaulting to \"{default}\".");
		default
	}));
	value_str.unwrap_or_else(|| {
		log!("Could not find entry '{key}' in 'engine config.hjson', defaulting to \"{default}\".");
		default
	})
}

pub fn read_hjson_i64(map: &Map<String, Value>, key: &'static str, default: i64) -> i64 {
	let value_str = map.get(key);
	let value_i64 = value_str.map(|v| v.as_i64().unwrap_or_else(|| {
		log!("Entry '{key}' in 'engine config.hjson' must be an int, defaulting to \"{default}\".");
		default
	}));
	value_i64.unwrap_or_else(|| {
		log!("Could not find entry '{key}' in 'engine config.hjson', defaulting to \"{default}\".");
		default
	})
}

pub fn read_hjson_f64(map: &Map<String, Value>, key: &'static str, default: f64) -> f64 {
	let value_str = map.get(key);
	let value_f64 = value_str.map(|v| v.as_f64().unwrap_or_else(|| {
		log!("Entry '{key}' in 'engine config.hjson' must be an int, defaulting to \"{default}\".");
		default
	}));
	value_f64.unwrap_or_else(|| {
		log!("Could not find entry '{key}' in 'engine config.hjson', defaulting to \"{default}\".");
		default
	})
}



pub fn load_uniform_datas(render_context: &wgpu_integration::RenderContextData) -> Result<UniformDatas> {
	
	let initial_data = Camera::default_data();
	let camera_buffer = render_context.device.create_buffer_init(
		&wgpu::util::BufferInitDescriptor {
			label: Some("Camera Buffer"),
			contents: bytemuck::cast_slice(&[initial_data]),
			usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
		}
	);
	
	let camera_bind_group_layout = render_context.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
		entries: &[
			wgpu::BindGroupLayoutEntry {
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
	
	let camera_bind_group = render_context.device.create_bind_group(&wgpu::BindGroupDescriptor {
		layout: &camera_bind_group_layout,
		entries: &[
			wgpu::BindGroupEntry {
				binding: 0,
				resource: camera_buffer.as_entire_binding(),
			}
		],
		label: Some("camera_bind_group"),
	});
	
	let depth_texture = wgpu_integration::create_depth_texture("depth_texture", render_context);
	
	Ok(UniformDatas {
		camera_binding: wgpu_integration::GeneralBindData {
			buffer: camera_buffer,
			layout: camera_bind_group_layout,
			group: camera_bind_group,
		},
		depth_texture,
	})
}



pub fn load_asset_datas(render_context: &wgpu_integration::RenderContextData) -> Result<AssetDatas> {
	
	let happy_tree_binding = wgpu_integration::load_texture("assets/happy-tree.png", render_context)?;
	
	Ok(AssetDatas {
		happy_tree_binding,
	})
}



pub fn load_world_datas(render_context: &wgpu_integration::RenderContextData) -> Result<WorldDatas> {
	
	let vertex_buffer = render_context.device.create_buffer_init(
		&wgpu::util::BufferInitDescriptor {
			label: Some("Vertex Buffer"),
			contents: bytemuck::cast_slice(VERTICES),
			usage: wgpu::BufferUsages::VERTEX,
		}
	);
	let index_buffer = render_context.device.create_buffer_init(
		&wgpu::util::BufferInitDescriptor {
			label: Some("Index Buffer"),
			contents: bytemuck::cast_slice(INDICES),
			usage: wgpu::BufferUsages::INDEX,
		}
	);
	
	let main_instances = (0..100).flat_map(|z| {
		(0..100).map(move |x| {
			let position = cgmath::Vector3 { x: x as f32, y: 0.0, z: z as f32 } - cgmath::Vector3::new(0.5, 0.0, 0.5);
			
			let rotation = if position.is_zero() {
				// this is needed so an object at (0, 0, 0) won't get scaled to zero
				// as Quaternions can affect scale if they're not created correctly
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
	
	let main_instances_data = main_instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
	let main_instances_buffer = render_context.device.create_buffer_init(
		&wgpu::util::BufferInitDescriptor {
			label: Some("Instance Buffer"),
			contents: bytemuck::cast_slice(&main_instances_data),
			usage: wgpu::BufferUsages::VERTEX,
		}
	);
	
	Ok(WorldDatas {
		
		main_vertices: vertex_buffer,
		main_indices: index_buffer,
		main_index_count: INDICES.len() as u32,
		
		main_instances,
		main_instances_buffer,
		
	})
}



pub fn load_pipelines(render_context: &wgpu_integration::RenderContextData, uniform_datas: &UniformDatas, asset_datas: &AssetDatas) -> Result<RenderPipelines> {
	
	// main pipeline
	let main = wgpu_integration::init_wgpu_pipeline(
		"Main",
		utils::get_program_file_path("shaders/main.wgsl"),
		&[
			&uniform_datas.camera_binding.layout,
			&asset_datas.happy_tree_binding.layout,
		],
		&[
			GenericVertex::get_layout(),
			InstanceRaw::get_layout()
		],
		render_context,
	)?;
	
	Ok(RenderPipelines {
		main
	})
}

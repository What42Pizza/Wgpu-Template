use crate::prelude::*;
use async_std::task::block_on;
use winit::window::Window;
use serde_hjson::{Map, Value};



pub mod load_generic_bind_layouts;
pub use load_generic_bind_layouts::*;
pub mod load_assets;
pub use load_assets::*;
pub mod load_pipelines;
pub use load_pipelines::*;





pub fn load_program_data(start_time: Instant, window: &Window) -> Result<ProgramData> {
	
	let engine_config = load_engine_config()?;
	
	// app data
	let camera_data = CameraData::new((0., 1., 2.));
	let shadow_caster_data = ShadowCasterData::default();
	let fps_counter = FpsCounter::new();
	
	// render data
	let render_context = load_render_context_data(window, &engine_config)?;
	let generic_bind_layouts = load_generic_bind_layouts(&render_context);
	let render_assets = load_render_assets(&camera_data, &shadow_caster_data, &render_context, &generic_bind_layouts, engine_config.shadowmap_size)?;
	let render_pipelines = load_render_pipelines(&render_context, &generic_bind_layouts, &render_assets)?;
	
	Ok(ProgramData {
		
		// engine data
		start_time,
		engine_config,
		pressed_keys: HashMap::new(),
		
		// app data
		camera_data,
		shadow_caster_data,
		fps_counter,
		
		// render data
		render_context,
		generic_bind_layouts,
		render_assets,
		render_pipelines,
		frame_start_instant: start_time,
		
	})
}





pub fn load_engine_config() -> Result<EngineConfig> {
	
	let engine_config_path = utils::get_program_file_path("engine config.hjson");
	let engine_config_result = fs::read_to_string(engine_config_path);
	let engine_config_string = match &engine_config_result {
		StdResult::Ok(v) => &**v,
		StdResult::Err(err) => {
			warn!("Failed to read 'engine config.hjson', using default values...  (error: {err})");
			include_str!("../../data/default engine config.hjson")
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
	
	let shadowmap_size_i64 = read_hjson_i64(&engine_config, "shadowmap_size", 512);
	let shadowmap_size = shadowmap_size_i64 as u32;
	
	Ok(EngineConfig {
		rendering_backend,
		present_mode,
		desired_frame_latency,
		min_frame_time,
		shadowmap_size,
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





pub fn load_render_context_data<'a>(window: &'a Window, engine_config: &load::EngineConfig) -> Result<RenderContextData<'a>> {
	block_on(load_render_context_data_async(window, engine_config))
}

pub async fn load_render_context_data_async<'a>(window: &'a Window, engine_config: &load::EngineConfig) -> Result<RenderContextData<'a>> {
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
		window,
		drawable_surface: surface,
		device,
		command_queue: queue,
		surface_config: config,
		surface_size: size,
		aspect_ratio: size.width as f32 / size.height as f32,
	})
}

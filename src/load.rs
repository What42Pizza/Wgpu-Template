use wgpu::util::DeviceExt;

use crate::prelude::*;



pub fn init_program_data(start_time: Instant, window: &Window) -> Result<ProgramData> {
	
	let camera = Camera::new((0., 1., 2.));
	let render_context = wgpu_integration::init_wgpu_context_data(window)?;
	let uniform_datas = load_uniform_datas(&render_context, &camera)?;
	let asset_datas = load_asset_datas(&render_context)?;
	let world_datas = load_world_datas(&render_context)?;
	let render_pipelines = load_pipelines(&render_context, &uniform_datas, &asset_datas, &world_datas)?;
	
	Ok(ProgramData {
		
		window,
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



pub fn load_uniform_datas(render_context: &wgpu_integration::RenderContextData, camera: &Camera) -> Result<UniformDatas> {
	
	let mut camera_data = wgpu_integration::CameraUniform::new();
	camera_data.update_view_proj(camera, render_context.aspect_ratio);
	
	let camera_buffer = render_context.device.create_buffer_init(
		&wgpu::util::BufferInitDescriptor {
			label: Some("Camera Buffer"),
			contents: bytemuck::cast_slice(&[camera_data]),
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
	
	Ok(UniformDatas {
		camera_data,
		camera_binding: wgpu_integration::BindingData {
			layout: camera_bind_group_layout,
			group: camera_bind_group,
		},
	})
}



pub fn load_asset_datas(render_context: &wgpu_integration::RenderContextData) -> Result<AssetDatas> {
	
	let happy_tree_binding = wgpu_integration::load_texture("assets/happy-tree.png", render_context)?;
	
	Ok(AssetDatas {
		happy_tree_binding,
	})
}



pub fn load_world_datas(render_context: &wgpu_integration::RenderContextData) -> Result<WorldDatas> {
	
	Ok(WorldDatas {
		
	})
}



pub fn load_pipelines(render_context: &wgpu_integration::RenderContextData, uniform_datas: &UniformDatas, asset_datas: &AssetDatas, world_datas: &WorldDatas) -> Result<(RenderPipelines)> {
	
	// main pipeline
	let main = wgpu_integration::init_wgpu_pipeline(
		"Main",
		utils::get_program_file_path("shaders/main.wgsl"),
		VERTICES,
		INDICES,
		&[
			&uniform_datas.camera_binding.layout,
			&asset_datas.happy_tree_binding.layout,
		],
		render_context,
	)?;
	
	Ok(RenderPipelines {
		main
	})
}

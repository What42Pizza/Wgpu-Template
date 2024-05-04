use crate::prelude::*;
use std::io::{BufReader, Cursor};
use wgpu::util::DeviceExt;



pub fn load_render_assets(
	camera_data: &CameraData,
	shadow_caster_data: &ShadowCasterData,
	render_context: &RenderContextData,
	generic_bind_layouts: &GenericBindLayouts,
	shadowmap_size: u32
) -> Result<RenderAssets> {
	
	let mut materials_storage = MaterialsStorage::new();
	let example_model = load_example_model_render_data(render_context, generic_bind_layouts, &mut materials_storage)?;
	let skybox_material_index = load_skybox_material(render_context, generic_bind_layouts, &mut materials_storage)?;
	let depth = load_depth_render_data(render_context)?;
	let shadow_caster = load_shadow_caster_data(render_context, shadowmap_size, shadow_caster_data)?;
	let camera = load_camera_render_data(render_context, camera_data)?;
	
	Ok(RenderAssets {
		materials_storage,
		example_models: example_model,
		skybox_material_id: skybox_material_index,
		depth,
		shadow_caster,
		camera,
	})
}





pub fn load_example_model_render_data(
	render_context: &RenderContextData,
	generic_bind_layouts: &GenericBindLayouts,
	materials_storage: &mut MaterialsStorage,
) -> Result<ModelsRenderData> {
	
	let example_model_meshes = load_model(utils::get_program_file_path("assets/cube.obj"), render_context, generic_bind_layouts, materials_storage)?;
	
	let example_model_instances = (0..100).flat_map(|z| {
		(0..100).map(move |x| {
			let position = glam::Vec3 { x: x as f32 * 3.0, y: 0.0, z: z as f32 * 3.0 } - glam::Vec3::new(0.5, 0.0, 0.5);
			
			let rotation = if position == glam::Vec3::ZERO {
				glam::Quat::from_axis_angle(glam::Vec3::Z, 0.0)
			} else {
				glam::Quat::from_axis_angle(position.normalize(), std::f32::consts::PI * 0.25)
			};
			
			InstanceData {
				position,
				rotation,
			}
		})
	}).collect::<Vec<_>>();
	
	let example_model_instances_data = example_model_instances.iter().map(InstanceData::to_raw).collect::<Vec<_>>();
	let example_model_instances_buffer = render_context.device.create_buffer_init(
		&wgpu::util::BufferInitDescriptor {
			label: Some("Instance Buffer"),
			contents: bytemuck::cast_slice(&example_model_instances_data),
			usage: wgpu::BufferUsages::VERTEX,
		}
	);
	
	Ok(ModelsRenderData {
		instances_buffer: example_model_instances_buffer,
		instances_count: example_model_instances.len() as u32,
		meshes: example_model_meshes,
	})
}



pub fn load_model(
	file_path: impl AsRef<Path>,
	render_context: &RenderContextData,
	generic_bind_layouts: &GenericBindLayouts,
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
		let path = parent_folder.join(&diffuse_texture_name);
		let material_id = match materials_storage_utils::get_material_id(&path, &materials_storage.list_2d) {
			Some(v) => v,
			None => materials_storage_utils::insert_material_2d(path, materials_storage, render_context, generic_bind_layouts)?,
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
				material_id: material_ids[model.mesh.material_id.unwrap_or(0)],
			}
		})
		.collect::<Vec<_>>();
	
	Ok(meshes)
}



pub fn load_skybox_material(render_context: &RenderContextData, generic_bind_layouts: &GenericBindLayouts, materials_storage: &mut MaterialsStorage) -> Result<usize> {
	materials_storage_utils::insert_material_cube(utils::get_program_file_path("assets/skybox.png"), materials_storage, render_context, generic_bind_layouts)
}





pub fn load_depth_render_data(render_context: &RenderContextData) -> Result<DepthRenderData> {
	
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
	
	Ok(DepthRenderData {
		view,
	})
}





pub fn load_shadow_caster_data(render_context: &RenderContextData, shadowmap_size: u32, shadow_caster_data: &ShadowCasterData) -> Result<ShadowCasterRenderData> {
	
	let size = wgpu::Extent3d {
		width: shadowmap_size,
		height: shadowmap_size,
		depth_or_array_layers: 1,
	};
	let desc = wgpu::TextureDescriptor {
		label: Some("Shadowmap Depth Texture"),
		size,
		mip_level_count: 1,
		sample_count: 1,
		dimension: wgpu::TextureDimension::D2,
		format: wgpu::TextureFormat::Depth32Float,
		usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
		view_formats: &[],
	};
	let depth_texture = render_context.device.create_texture(&desc);
	let depth_tex_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());
	
	let proj_mat_layout = render_context.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
			},
		],
		label: Some("shadow_caster_bind_group_layout"),
	});
	
	let proj_mat_buffer = render_context.device.create_buffer_init(
		&wgpu::util::BufferInitDescriptor {
			label: Some("Shadow Caster Buffer"),
			contents: bytemuck::cast_slice(&shadow_caster_data.build_gpu_data()),
			usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
		}
	);
	
	let proj_mat_group = render_context.device.create_bind_group(&wgpu::BindGroupDescriptor {
		layout: &proj_mat_layout,
		entries: &[
			wgpu::BindGroupEntry { // proj_mat
				binding: 0,
				resource: proj_mat_buffer.as_entire_binding(),
			}
		],
		label: Some("shadow_caster_bind_group"),
	});
	
	Ok(ShadowCasterRenderData {
		is_dirty: false,
		depth_tex_view,
		proj_mat_buffer,
		proj_mat_layout,
		proj_mat_group,
	})
}





pub fn load_camera_render_data(render_context: &RenderContextData, camera_data: &CameraData) -> Result<CameraRenderData> {
	
	let bind_layout = render_context.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
		entries: &[
			wgpu::BindGroupLayoutEntry { // proj_view_mat, inv_proj_mat, view_mat
				binding: 0,
				visibility: wgpu::ShaderStages::VERTEX,
				ty: wgpu::BindingType::Buffer {
					ty: wgpu::BufferBindingType::Uniform,
					has_dynamic_offset: false,
					min_binding_size: None,
				},
				count: None,
			},
		],
		label: Some("camera_bind_group_layout"),
	});
	
	let buffer = render_context.device.create_buffer_init(
		&wgpu::util::BufferInitDescriptor {
			label: Some("Camera Buffer"),
			contents: bytemuck::cast_slice(&camera_data.build_gpu_data(render_context.aspect_ratio)),
			usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
		}
	);
	
	let bind_group = render_context.device.create_bind_group(&wgpu::BindGroupDescriptor {
		layout: &bind_layout,
		entries: &[
			wgpu::BindGroupEntry { // proj_view_mat, inv_proj_mat, view_mat
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

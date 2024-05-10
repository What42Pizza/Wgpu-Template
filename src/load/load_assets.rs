use crate::prelude::*;
use std::io::{BufReader, Cursor};
use wgpu::util::DeviceExt;



pub fn load_render_assets(
	camera_data: &CameraData,
	shadow_caster_data: &ShadowCasterData,
	render_context: &RenderContextData,
	shadowmap_size: u32,
	binding_1_layout: &wgpu::BindGroupLayout,
) -> Result<RenderAssets> {
	
	let mut materials_storage = MaterialsStorage::new();
	let example_models = load_example_models_render_data(render_context, &mut materials_storage, binding_1_layout).context("Failed to load model render data.")?;
	let skybox_material_index = load_skybox_material(render_context, &mut materials_storage).context("Failed to load skybox render data.")?;
	let depth = load_depth_render_data(render_context);
	let shadow_caster = load_shadow_caster_data(render_context, shadowmap_size, shadow_caster_data, camera_data).context("Failed to load shadow caster render data.")?;
	let camera = load_camera_render_data(render_context, camera_data).context("Failed to load camera render data.")?;
	
	Ok(RenderAssets {
		materials_storage,
		example_models,
		skybox_material_id: skybox_material_index,
		depth,
		shadow_caster,
		camera,
	})
}





pub fn load_example_models_render_data(
	render_context: &RenderContextData,
	materials_storage: &mut MaterialsStorage,
	binding_1_layout: &wgpu::BindGroupLayout,
) -> Result<ModelsRenderData> {
	
	let example_model_meshes = load_model(utils::get_program_file_path("assets/cube.obj"), render_context, materials_storage, binding_1_layout)?;
	
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
	materials_storage: &mut MaterialsStorage,
	binding_1_layout: &wgpu::BindGroupLayout,
) -> Result<Vec<MeshRenderData>> {
	let file_path = file_path.as_ref();
	let obj_text = fs::read_to_string(file_path).add_path_to_error(&file_path)?;
	let obj_cursor = Cursor::new(obj_text);
	let mut obj_reader = BufReader::new(obj_cursor);
	let parent_folder = file_path.parent().expect("Cannot load mesh at root directory");
	
	let (models, model_materials) = tobj::load_obj_buf(
		&mut obj_reader,
		&tobj::LoadOptions {
			triangulate: true,
			single_index: true,
			..Default::default()
		},
		move |p| {
			let mat_text =
				fs::read_to_string(parent_folder.join(p))
				//.add_path_to_error(&file_path) // this would just be thrown away in the next map_err()
				.map_err(|_err| tobj::LoadError::OpenFileFailed)?;
			tobj::load_mtl_buf(&mut BufReader::new(Cursor::new(mat_text)))
		}
	).context("Failed to decode model data.")?;
	let model_materials = model_materials.context("Failed to read model materials")?;
	
	let mut material_ids = Vec::new();
	for material in model_materials {
		let Some(diffuse_texture_name) = material.diffuse_texture else {
			warn!("diffuse texture in material is `None`.");
			continue;
		};
		let path = parent_folder.join(&diffuse_texture_name);
		let material_id = match materials_storage_utils::get_material_id(&path, &materials_storage.list_2d) {
			Some(v) => v,
			None => materials_storage_utils::insert_material_2d(path, materials_storage, render_context)?,
		};
		material_ids.push(material_id);
	}
	
	let meshes = models
		.into_iter()
		.enumerate()
		.map(|(i, model)| {
			let pos_count = model.mesh.positions.len() / 3;
			let mut basic_vertices = Vec::with_capacity(pos_count);
			let mut extended_vertices = Vec::with_capacity(pos_count);
			for j in 0..pos_count {
				basic_vertices.push(BasicVertexData {
					position: [
						model.mesh.positions[j * 3],
						model.mesh.positions[j * 3 + 1],
						model.mesh.positions[j * 3 + 2],
					],
				});
				extended_vertices.push(ExtendedVertexData {
					tex_coords: [model.mesh.texcoords[j * 2], 1.0 - model.mesh.texcoords[j * 2 + 1]],
					normal: [
						model.mesh.normals[j * 3],
						model.mesh.normals[j * 3 + 1],
						model.mesh.normals[j * 3 + 2],
					],
				});
			}
			
			let basic_vertex_buffer = render_context.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
				label: Some(&format!("{:?} Basic Vertex Buffer", &file_path)),
				contents: bytemuck::cast_slice(&basic_vertices),
				usage: wgpu::BufferUsages::VERTEX,
			});
			let extended_vertex_buffer = render_context.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
				label: Some(&format!("{:?} Extended Vertex Buffer", &file_path)),
				contents: bytemuck::cast_slice(&extended_vertices),
				usage: wgpu::BufferUsages::VERTEX,
			});
			let index_buffer = render_context.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
				label: Some(&format!("{:?} Index Buffer", &file_path)),
				contents: bytemuck::cast_slice(&model.mesh.indices),
				usage: wgpu::BufferUsages::INDEX,
			});
			
			let material_id = material_ids[model.mesh.material_id.unwrap_or(0)];
			let material_view = &materials_storage.list_2d[material_id].view;
			let binding_1 = render_context.device.create_bind_group(&wgpu::BindGroupDescriptor {
				label: Some(&format!("{:?} Binding", &file_path)),
				layout: binding_1_layout,
				entries: &[
					wgpu::BindGroupEntry {
						binding: 0,
						resource: wgpu::BindingResource::TextureView (material_view),
					},
				],
			});
			
			MeshRenderData {
				basic_vertex_buffer,
				extended_vertex_buffer,
				index_buffer,
				index_count: model.mesh.indices.len() as u32,
				binding_1,
			}
		})
		.collect::<Vec<_>>();
	
	Ok(meshes)
}



pub fn load_skybox_material(render_context: &RenderContextData, materials_storage: &mut MaterialsStorage) -> Result<usize> {
	materials_storage_utils::insert_material_cube(utils::get_program_file_path("assets/skybox.png"), materials_storage, render_context)
}





pub fn load_depth_render_data(render_context: &RenderContextData) -> DepthRenderData {
	
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
	
	DepthRenderData {
		view,
	}
}





pub fn load_shadow_caster_data(render_context: &RenderContextData, shadowmap_size: u32, shadow_caster_data: &ShadowCasterData, camera_data: &CameraData) -> Result<ShadowCasterRenderData> {
	
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
	let depth_tex_view = depth_texture.create_view(&wgpu::TextureViewDescriptor {
		label: None,
		format: None,
		dimension: Some(wgpu::TextureViewDimension::D2),
		aspect: wgpu::TextureAspect::All,
		base_mip_level: 0,
		mip_level_count: None,
		base_array_layer: 0,
		array_layer_count: None,
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
	
	let proj_mat_buffer = render_context.device.create_buffer_init(
		&wgpu::util::BufferInitDescriptor {
			label: Some("Shadow Caster Buffer"),
			contents: bytemuck::cast_slice(&shadow_caster_data.build_gpu_data(camera_data.pos)),
			usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
		}
	);
	
	Ok(ShadowCasterRenderData {
		depth_tex_view,
		proj_mat_buffer,
	})
}





pub fn load_camera_render_data(render_context: &RenderContextData, camera_data: &CameraData) -> Result<CameraRenderData> {
	
	let buffer = render_context.device.create_buffer_init(
		&wgpu::util::BufferInitDescriptor {
			label: Some("Camera Buffer"),
			contents: bytemuck::cast_slice(&camera_data.build_gpu_data(render_context.aspect_ratio)),
			usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
		}
	);
	
	Ok(CameraRenderData {
		buffer,
	})
}

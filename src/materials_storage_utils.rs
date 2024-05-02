use crate::prelude::*;



pub fn get_material_index(path: impl AsRef<Path>, list: &[MaterialRenderData]) -> Option<usize> {
	let path = path.as_ref();
	list.iter().enumerate()
		.find(|(_i, material)| &*material.path == path)
		.map(|(i, _material)| i)
}

pub fn insert_material_2d(
	path: impl Into<PathBuf>,
	materials_storage: &mut MaterialsStorage,
	render_context: &RenderContextData,
	generic_texture_bind_layout: &wgpu::BindGroupLayout,
) -> Result<usize> {
	let output = materials_storage.list_2d.len();
	let material = load_material_2d(path, render_context, generic_texture_bind_layout)?;
	materials_storage.list_2d.push(material);
	Ok(output)
}

pub fn insert_material_cube(
	path: impl Into<PathBuf>,
	materials_storage: &mut MaterialsStorage,
	render_context: &RenderContextData,
	cube_texture_bind_layout: &wgpu::BindGroupLayout,
) -> Result<usize> {
	let output = materials_storage.list_cube.len();
	let material = load_material_cube(path, render_context, cube_texture_bind_layout)?;
	materials_storage.list_cube.push(material);
	Ok(output)
}





// WARNING: This is only meant to be used by 'load_material_to_storage'. Loading materials with this manually could lead to several copies of the same image, which is wasteful
pub fn load_material_2d(
	path: impl Into<PathBuf>,
	render_context: &RenderContextData,
	texture_bind_layout: &wgpu::BindGroupLayout,
) -> Result<MaterialRenderData> {
	let path = path.into();
	
	let raw_texture_bytes = fs::read(utils::get_program_file_path(&path))?;
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
			label: None,
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
	
	let bind_group = render_context.device.create_bind_group(&wgpu::BindGroupDescriptor {
		layout: texture_bind_layout,
		entries: &[
			wgpu::BindGroupEntry { // texture view
				binding: 0,
				resource: wgpu::BindingResource::TextureView(&view),
			},
			wgpu::BindGroupEntry { // sampler
				binding: 1,
				resource: wgpu::BindingResource::Sampler(&sampler),
			},
		],
		label: None,
	});
	
	Ok(MaterialRenderData {
		path,
		bind_group,
	})
}



// WARNING: This is only meant to be used by 'load_material_to_storage'. Loading materials with this manually could lead to several copies of the same image, which is wasteful
pub fn load_material_cube(
	path: impl Into<PathBuf>,
	render_context: &RenderContextData,
	texture_bind_layout: &wgpu::BindGroupLayout,
) -> Result<MaterialRenderData> {
	let path = path.into();
	
	let raw_texture_bytes = fs::read(utils::get_program_file_path(&path))?;
	let texture_bytes = image::load_from_memory(&raw_texture_bytes)?;
	let texture_bytes = texture_bytes.to_rgba8();
	let dimensions = texture_bytes.dimensions();
	let dimensions = (dimensions.0, dimensions.1 / 6);
	
	let texture_size = wgpu::Extent3d {
		width: dimensions.0,
		height: dimensions.1,
		depth_or_array_layers: 6,
	};
	let texture = render_context.device.create_texture(
		&wgpu::TextureDescriptor {
			size: texture_size,
			mip_level_count: 1,
			sample_count: 1,
			dimension: wgpu::TextureDimension::D2,
			format: wgpu::TextureFormat::Rgba8UnormSrgb,
			usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
			label: None,
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
	
	let view = texture.create_view(&wgpu::TextureViewDescriptor {
		dimension: Some(wgpu::TextureViewDimension::Cube),
		..Default::default()
	});
	let sampler = render_context.device.create_sampler(&wgpu::SamplerDescriptor {
		address_mode_u: wgpu::AddressMode::ClampToEdge,
		address_mode_v: wgpu::AddressMode::ClampToEdge,
		address_mode_w: wgpu::AddressMode::ClampToEdge,
		mag_filter: wgpu::FilterMode::Linear,
		min_filter: wgpu::FilterMode::Nearest,
		mipmap_filter: wgpu::FilterMode::Nearest,
		..Default::default()
	});
	
	let bind_group = render_context.device.create_bind_group(&wgpu::BindGroupDescriptor {
		layout: texture_bind_layout,
		entries: &[
			wgpu::BindGroupEntry { // texture view
				binding: 0,
				resource: wgpu::BindingResource::TextureView(&view),
			},
			wgpu::BindGroupEntry { // sampler
				binding: 1,
				resource: wgpu::BindingResource::Sampler(&sampler),
			},
		],
		label: None,
	});
	
	Ok(MaterialRenderData {
		path,
		bind_group,
	})
}

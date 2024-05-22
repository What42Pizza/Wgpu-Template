use crate::prelude::*;

// HELP: The purpose of this is to make sure textures are only loaded once



pub fn get_material_id(path: impl AsRef<Path>, list: &[MaterialRenderData]) -> Option<MaterialId> {
	let path = path.as_ref();
	list.iter().enumerate()
		.find(|(_i, material)| &*material.path == path)
		.map(|(i, _material)| i)
}

pub fn insert_material_2d(
	path: impl Into<PathBuf>,
	materials_storage: &mut MaterialsStorage,
	render_context: &RenderContextData,
	compress_textures: bool,
) -> Result<MaterialId> {
	let output = materials_storage.list_2d.len();
	let material = load_material_2d(path, render_context, compress_textures).context("Failed to load material_2d.")?;
	materials_storage.list_2d.push(material);
	Ok(output)
}

pub fn insert_material_cube(
	path: impl Into<PathBuf>,
	materials_storage: &mut MaterialsStorage,
	render_context: &RenderContextData,
	compress_textures: bool,
) -> Result<MaterialId> {
	let output = materials_storage.list_cube.len();
	let material = load_material_cube(path, render_context, compress_textures).context("Failed to load material_cube")?;
	materials_storage.list_cube.push(material);
	Ok(output)
}





// WARNING: This is only meant to be used by 'load_material_to_storage'. Loading materials with this manually could lead to several copies of the same image, which is wasteful
pub fn load_material_2d(
	path: impl Into<PathBuf>,
	render_context: &RenderContextData,
	compress_textures: bool,
) -> Result<MaterialRenderData> {
	let path = path.into();
	
	let raw_texture_bytes = fs::read(utils::get_program_file_path(&path)).add_path_to_error(&path)?;
	let texture_bytes = image::load_from_memory(&raw_texture_bytes).context("Failed to decode texture.")?;
	let texture_bytes = texture_bytes.to_rgba8();
	let dimensions = texture_bytes.dimensions();
	let mut texture_bytes = texture_bytes.into_raw();
	
	if compress_textures {
		let compress_settings = intel_tex_2::bc7::opaque_fast_settings();
		texture_bytes = intel_tex_2::bc7::compress_blocks(&compress_settings, &intel_tex_2::Surface {
			data: &*texture_bytes,
			width: dimensions.0,
			height: dimensions.1,
			stride: dimensions.0 * 4,
		});
	}
	
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
			format: wgpu::TextureFormat::Bc7RgbaUnormSrgb,
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
	
	Ok(MaterialRenderData {
		path,
		view,
	})
}



// WARNING: This is only meant to be used by 'load_material_to_storage'. Loading materials with this manually could lead to several copies of the same image, which is wasteful
pub fn load_material_cube(
	path: impl Into<PathBuf>,
	render_context: &RenderContextData,
	compress_textures: bool,
) -> Result<MaterialRenderData> {
	let path = path.into();
	
	let raw_texture_bytes = fs::read(utils::get_program_file_path(&path)).add_path_to_error(&path)?;
	let texture_bytes = image::load_from_memory(&raw_texture_bytes).context("Failed to decode texture.")?;
	let texture_bytes = texture_bytes.to_rgba8();
	let dimensions = texture_bytes.dimensions();
	let mut texture_bytes = texture_bytes.into_raw();
	
	if compress_textures {
		let compress_settings = intel_tex_2::bc7::opaque_fast_settings();
		texture_bytes = intel_tex_2::bc7::compress_blocks(&compress_settings, &intel_tex_2::Surface {
			data: &*texture_bytes,
			width: dimensions.0,
			height: dimensions.1,
			stride: dimensions.0 * 4,
		});
	}
	
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
	
	Ok(MaterialRenderData {
		path,
		view,
	})
}

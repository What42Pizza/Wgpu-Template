//! A 'render module' here is a system that renders something in a specific way.
//! 
//! There are currently four render modules:
//! - models_main: renders generic models to the screen
//! - models_shadowmap: renders generic models to the shadowmap
//! - skybox: renders a cubic skybox to the screen
//! - post_processing: alters another texture with basic color processing



use crate::*;

pub mod models_main;
pub mod models_shadowmap;
pub mod skybox;
pub mod post_processing;



pub struct AllModuleLayouts {
	pub models_main:      models_main::ModelsMainLayouts,
	pub models_shadowmap: models_shadowmap::ModelsShadowmapLayouts,
	pub skybox:           skybox::SkyboxLayouts,
	pub post_processing:  post_processing::PostProcessingLayouts,
}

pub struct AllModuleBindings {
	pub models_main:      models_main::ModelsMainBindings,
	pub models_shadowmap: models_shadowmap::ModelsShadowmapBindings,
	pub skybox:           skybox::SkyboxBindings,
	pub post_processing:  post_processing::PostProcessingBindings,
}



pub fn load_all_module_layouts(render_context: &RenderContext) -> Result<AllModuleLayouts> {
	Ok(AllModuleLayouts {
		models_main:      models_main::load_layout(render_context)?,
		models_shadowmap: models_shadowmap::load_layout(render_context)?,
		skybox:           skybox::load_layout(render_context)?,
		post_processing:  post_processing::load_layout(render_context)?,
	})
}

pub fn load_all_module_bindings(render_context: &RenderContext, render_layouts: &AllModuleLayouts, render_assets: &RenderAssets) -> AllModuleBindings {
	AllModuleBindings {
		models_main:      models_main::     load_bindings(render_context, &render_layouts.models_main     , render_assets),
		models_shadowmap: models_shadowmap::load_bindings(render_context, &render_layouts.models_shadowmap, render_assets),
		skybox:           skybox::          load_bindings(render_context, &render_layouts.skybox          , render_assets),
		post_processing:  post_processing:: load_bindings(render_context, &render_layouts.post_processing , render_assets),
	}
}



pub fn render(output: &wgpu::SurfaceTexture, program_data: &mut ProgramData) {
	
	let output_view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
	let encoder_descriptor = wgpu::CommandEncoderDescriptor {label: None};
	let mut encoder = program_data.render_context.gpu_device.create_command_encoder(&encoder_descriptor);
	
	models_main::     render(&program_data.module_layouts.models_main     , &program_data.render_assets, &program_data.module_bindings.models_main     , &mut encoder);
	models_shadowmap::render(&program_data.module_layouts.models_shadowmap, &program_data.render_assets, &program_data.module_bindings.models_shadowmap, &mut encoder);
	skybox::          render(&program_data.module_layouts.skybox          , &program_data.render_assets, &program_data.module_bindings.skybox          , &mut encoder); // HELP: it's better to have this at the end so that only the necessary pixels are rendered
	post_processing:: render(&program_data.module_layouts.post_processing , &program_data.render_assets, &program_data.module_bindings.post_processing , &mut encoder, &output_view);
	
	program_data.render_context.gpu_command_queue.submit(std::iter::once(encoder.finish()));
}

use crate::prelude::*;
use winit::{dpi::PhysicalPosition, keyboard::KeyCode};



pub struct ProgramData<'a> {
	
	// engine data
	pub start_time: Instant,
	pub engine_config: EngineConfig,
	pub input: EngineInput,
	
	// app data
	pub camera_data: CameraData,
	pub shadow_caster_data: ShadowCasterData,
	pub example_model_instance_datas: Vec<InstanceData>,
	pub fps_counter: FpsCounter,
	pub is_moving_camera: bool,
	pub color_correction_settings: ColorCorrectionSettings,
	
	// render data
	pub render_context: RenderContextData<'a>,
	// HELP: There are (currently) three render 'modules': shadow_caster, models, and
	// skybox. The layouts for all three are created, then the assets (buffers, tex
	// views, etc) for all three are created, then the bindings to the assets for all
	// three are created.
	pub render_layouts: RenderLayouts,
	pub render_assets: RenderAssets,
	pub render_bindings: RenderBindings,
	pub frame_start_instant: Instant,
	
}

impl<'a> ProgramData<'a> {
	pub fn step_dt(&mut self) -> f32 {
		let new_frame_instant = Instant::now();
		let dt = (new_frame_instant - self.frame_start_instant).as_secs_f32();
		self.frame_start_instant = new_frame_instant;
		dt
	}
}





pub struct EngineConfig {
	pub rendering_backend: wgpu::Backends,
	pub present_mode: wgpu::PresentMode,
	pub desired_frame_latency: u32,
	pub min_frame_time: Duration,
	pub shadowmap_size: u32,
}



pub struct EngineInput {
	pub window_is_focused: bool,
	pub pressed_keys: HashSet<KeyCode>,
	pub prev_pressed_keys: HashSet<KeyCode>,
	pub mouse_pos: PhysicalPosition<f64>,
	pub prev_mouse_pos: PhysicalPosition<f64>,
	pub pressed_mouse_buttons: PressedMouseButtons,
	pub prev_pressed_mouse_buttons: PressedMouseButtons,
}

impl EngineInput {
	pub fn key_is_down(&self, key: KeyCode) -> bool {
		self.pressed_keys.contains(&key)
	}
	pub fn key_just_pressed(&self, key: KeyCode) -> bool {
		self.pressed_keys.contains(&key) && !self.prev_pressed_keys.contains(&key)
	}
	pub fn button_just_pressed(&self, button: MouseButton) -> bool {
		match button {
			MouseButton::Left    => self.pressed_mouse_buttons.left_is_down    && !self.prev_pressed_mouse_buttons.left_is_down,
			MouseButton::Right   => self.pressed_mouse_buttons.right_is_down   && !self.prev_pressed_mouse_buttons.right_is_down,
			MouseButton::Middle  => self.pressed_mouse_buttons.middle_is_down  && !self.prev_pressed_mouse_buttons.middle_is_down,
			MouseButton::Back    => self.pressed_mouse_buttons.back_is_down    && !self.prev_pressed_mouse_buttons.back_is_down,
			MouseButton::Forward => self.pressed_mouse_buttons.forward_is_down && !self.prev_pressed_mouse_buttons.forward_is_down,
			MouseButton::Other (id) => self.pressed_mouse_buttons.others_down.contains(&id) && !self.prev_pressed_mouse_buttons.others_down.contains(&id),
		}
	}
}



#[derive(Default, Clone)]
pub struct PressedMouseButtons {
	pub left_is_down: bool,
	pub right_is_down: bool,
	pub middle_is_down: bool,
	pub back_is_down: bool,
	pub forward_is_down: bool,
	pub others_down: HashSet<u16>,
}





pub struct CameraData {
	pub pos: glam::Vec3,
	pub rot_xz: f32,
	pub rot_y: f32,
	pub fov_radians: f32,
	pub near: f32,
	pub far: f32,
}

impl CameraData {
	// HELP: Ideally you should use some sort of processing cpu-side that accounts for
	// the fact that `glam` (and similar crates) expect a z-range of -1 to 1 while wgpu
	// expects a z-range of 0 to 1, but I haven't been able to integrate this matrix with
	// the skybox code, and I've found that it's easier to just correct the z-range at
	// the end of the vertex shaders (`pos.z = pos.z * 0.5 + 0.5`)
	pub const OPENGL_TO_WGPU_MATRIX: glam::Mat4 = glam::Mat4::from_cols_array(&[
		1.0, 0.0, 0.0, 0.0,
		0.0, 1.0, 0.0, 0.0,
		0.0, 0.0, 0.5, 0.5,
		0.0, 0.0, 0.0, 1.0,
	]);
	pub fn build_gpu_data(&self, aspect_ratio: f32) -> [f32; 16 + 16 + 16] {
		let proj = glam::Mat4::perspective_rh(self.fov_radians, aspect_ratio, self.near, self.far);
		let target = self.pos + glam::Vec3::new(
			self.rot_xz.cos() * self.rot_y.cos(),
			self.rot_y.sin(),
			self.rot_xz.sin() * self.rot_y.cos(),
		);
		let view = glam::Mat4::look_at_rh(self.pos, target, glam::Vec3::Y);
		let inv_proj = proj.inverse();
		let proj_view = proj * view;
		let mut output = [0f32; 16 + 16 + 16];
		output[..16].copy_from_slice(&proj_view.to_cols_array());
		output[16..32].copy_from_slice(&inv_proj.to_cols_array());
		output[32..48].copy_from_slice(&view.to_cols_array());
		output
	}
	pub fn new(pos: (f32, f32, f32)) -> Self {
		Self {
			pos: pos.into(),
			rot_xz: 0.0,
			rot_y: 0.0,
			fov_radians: 70.0f32.to_radians(),
			near: 0.1,
			far: 500.0,
		}
	}
}



pub struct ShadowCasterData {
	pub size: glam::Vec3,
	pub rot: glam::Quat,
}

impl ShadowCasterData {
	pub fn build_gpu_data(&self, center_pos: glam::Vec3) -> [f32; 16] {
		let trans_mat = glam::Mat4::from_translation(-center_pos);
		let rot_mat = glam::Mat4::from_quat(self.rot);
		let scale_mat = glam::Mat4::from_scale(1.0 / self.size);
		let output = scale_mat * rot_mat * trans_mat;
		output.to_cols_array()
	}
}

impl Default for ShadowCasterData {
	fn default() -> Self {
		const PI: f32 = std::f32::consts::PI;
		Self {
			size: glam::Vec3::new(200.0, 200.0, 200.0),
			rot: glam::Quat::from_euler(glam::EulerRot::YXZ, PI * -0.25, PI * -0.125, 0.0),
		}
	}
}



pub struct FpsCounter {
	pub frame_count: usize,
	pub frame_time_total: Duration,
	pub next_output_time: Instant,
}

impl FpsCounter {
	
	pub fn new() -> Self {
		Self {
			frame_count: 0,
			frame_time_total: Duration::ZERO,
			next_output_time: Instant::now(),
		}
	}
	
	pub fn step(&mut self, frame_time: Duration) -> Option<(usize, Duration)> {
		
		self.frame_count += 1;
		self.frame_time_total += frame_time;
		if self.next_output_time.elapsed().as_secs_f32() < 1.0 {return None;}
		
		let fps_output = self.frame_count;
		let duration_output = self.frame_time_total / self.frame_count as u32;
		
		self.frame_count = 0;
		self.frame_time_total = Duration::ZERO;
		self.next_output_time += Duration::SECOND;
		
		Some((fps_output, duration_output))
	}
	
}



#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ColorCorrectionSettings {
	pub saturation: f32,
	pub brightness: f32,
}

impl Default for ColorCorrectionSettings {
	fn default() -> Self {
		Self {
			saturation: 1.1,
			brightness: 1.1,
		}
	}
}





pub struct RenderContextData<'a> {
	pub window: &'a Window,
	pub drawable_surface: wgpu::Surface<'a>,
	pub device: wgpu::Device,
	pub command_queue: wgpu::Queue,
	pub surface_config: wgpu::SurfaceConfiguration,
	pub surface_size: winit::dpi::PhysicalSize<u32>,
	pub surface_format: wgpu::TextureFormat,
	pub aspect_ratio: f32,
}



pub struct RenderLayouts {
	
	// shadow_caster render data
	pub shadow_caster_pipeline: wgpu::RenderPipeline,
	pub shadow_caster_bind_0_layout: wgpu::BindGroupLayout,
	
	// models render data
	pub models_pipeline: wgpu::RenderPipeline,
	pub models_bind_0_layout: wgpu::BindGroupLayout,
	pub models_bind_1_layout: wgpu::BindGroupLayout,
	
	// skybox render data
	pub skybox_pipeline: wgpu::RenderPipeline,
	pub skybox_bind_0_layout: wgpu::BindGroupLayout,
	
	// color correction data
	pub color_correction_pipeline: wgpu::RenderPipeline,
	pub color_correction_bind_0_layout: wgpu::BindGroupLayout,
	
}



// this holds buffers, views, samplers, etc
pub struct RenderAssets {
	
	// general render data
	pub depth: DepthRenderData,
	pub main_tex_view: wgpu::TextureView,
	pub camera: CameraRenderData,
	pub default_sampler: wgpu::Sampler,
	pub materials_storage: MaterialsStorage,
	
	// shadow_caster render data
	pub shadow_caster: ShadowCasterRenderData,
	
	// models render data
	pub example_models: ModelsRenderData,
	
	// skybox render data
	pub skybox_material_id: MaterialId,
	pub skybox_sampler: wgpu::Sampler,
	
	// color correction data
	pub color_correction_buffer: wgpu::Buffer,
	
}

pub struct MaterialsStorage {
	pub list_2d: Vec<MaterialRenderData>,
	pub list_cube: Vec<MaterialRenderData>,
}

impl MaterialsStorage {
	pub fn new() -> Self {
		Self {
			list_2d: vec!(),
			list_cube: vec!(),
		}
	}
}

pub type MaterialId = usize;

pub struct MaterialRenderData {
	// `path` is used to make sure the same data isn't loaded multiple times
	pub path: PathBuf,
	pub view: wgpu::TextureView,
}

pub struct ModelsRenderData {
	// defines the data per model
	pub culled_instances_buffer: wgpu::Buffer,
	pub culled_instances_count: u32,
	pub total_instances_buffer: wgpu::Buffer,
	pub total_instances_count: u32,
	pub bounding_radius: f32,
	pub meshes: Vec<MeshRenderData>, // defines the data for a single model
}

pub struct MeshRenderData {
	pub basic_vertex_buffer: wgpu::Buffer,
	pub extended_vertex_buffer: wgpu::Buffer,
	pub index_buffer: wgpu::Buffer,
	pub index_count: u32,
	pub material_id: MaterialId,
}

// HELP: Many structs like this only have whatever data is actually used, if you run into
// a situation where you also need the Texture, Sampler, etc then you can just add them
// to the relevant struct
pub struct DepthRenderData {
	pub view: wgpu::TextureView,
}

pub struct ShadowCasterRenderData {
	pub depth_tex_view: wgpu::TextureView,
	pub depth_sampler: wgpu::Sampler,
	pub proj_mat_buffer: wgpu::Buffer,
}

// HELP: It may be a bit disorienting to have two Camera structs, but just keep this is
// mind: the struct `CameraData` holds the data used for app logic, the struct
// `CameraRenderData` holds the data for rendering logic, and data is moved from `Camera`
// to `CameraRenderData` each frame (or whenever needed)
pub struct CameraRenderData {
	pub buffer: wgpu::Buffer,
}



pub struct RenderBindings {
	
	// shadow_caster render data
	pub shadow_caster_bind_0: wgpu::BindGroup,
	
	// models render data
	pub models_bind_0: wgpu::BindGroup,
	pub example_models_bind_1s: Vec<wgpu::BindGroup>, // corresponds to the vec in render_assets.example_models.meshes
	
	// skybox render data
	pub skybox_bind_0: wgpu::BindGroup,
	
	// color correction data
	pub color_correction_bind_0: wgpu::BindGroup,
	
}



#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BasicVertexData {
	pub pos: [f32; 3],
}

impl BasicVertexData {
	pub const ATTRIBUTES: [wgpu::VertexAttribute; 1] = wgpu::vertex_attr_array![
		0 => Float32x3,
	];
	pub const fn get_layout() -> wgpu::VertexBufferLayout<'static> {
		wgpu::VertexBufferLayout {
			array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
			step_mode: wgpu::VertexStepMode::Vertex,
			attributes: &Self::ATTRIBUTES,
		}
	}
}



// NOTE: 'extended' here means more advanced, having more data
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ExtendedVertexData {
	pub tex_coords: [f32; 2],
	pub normal: [f32; 3],
}

impl ExtendedVertexData {
	pub const ATTRIBUTES: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![
		1 => Float32x2,
		2 => Float32x3,
	];
	pub const fn get_layout() -> wgpu::VertexBufferLayout<'static> {
		wgpu::VertexBufferLayout {
			array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
			step_mode: wgpu::VertexStepMode::Vertex,
			attributes: &Self::ATTRIBUTES,
		}
	}
}



pub struct InstanceData {
	pub pos: glam::Vec3,
	pub rot: glam::Quat,
}

impl InstanceData {
	pub fn to_raw(&self) -> RawInstanceData {
		let model_data = glam::Mat4::from_translation(self.pos) * glam::Mat4::from_quat(self.rot);
		RawInstanceData {
			model: model_data.to_cols_array_2d(),
		}
	}
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RawInstanceData {
	pub model: [[f32; 4]; 4],
}

impl RawInstanceData {
	pub const ATTRIBUTES: [wgpu::VertexAttribute; 4] = wgpu::vertex_attr_array![
		3 => Float32x4,
		4 => Float32x4,
		5 => Float32x4,
		6 => Float32x4
	];
	pub const fn get_layout() -> wgpu::VertexBufferLayout<'static> {
		use std::mem;
		wgpu::VertexBufferLayout {
			array_stride: mem::size_of::<RawInstanceData>() as wgpu::BufferAddress,
			step_mode: wgpu::VertexStepMode::Instance,
			attributes: &Self::ATTRIBUTES,
		}
	}
}

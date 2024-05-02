use crate::prelude::*;
use cgmath::SquareMatrix;
use winit::keyboard::KeyCode;



pub struct ProgramData<'a> {
	
	pub start_time: Instant,
	pub window: &'a Window,
	pub pressed_keys: HashMap<KeyCode, bool>,
	pub frame_start_instant: Instant,
	pub min_frame_time: Duration,
	pub fps_counter: FpsCounter,
	
	pub render_context: wgpu_integration::RenderContextData<'a>,
	pub render_layouts: TextureLayouts,
	pub render_assets: RenderAssets,
	pub render_pipelines: RenderPipelines,
	
	pub camera: Camera,
	
}

impl<'a> ProgramData<'a> {
	pub fn key_is_down(&self, key: KeyCode) -> bool {
		self.pressed_keys.get(&key).cloned().unwrap_or(false)
	}
	pub fn step_dt(&mut self) -> f32 {
		let new_frame_instant = Instant::now();
		let dt = (new_frame_instant - self.frame_start_instant).as_secs_f32();
		self.frame_start_instant = new_frame_instant;
		dt
	}
}





pub struct TextureLayouts {
	pub generic: wgpu::BindGroupLayout,
	pub cube: wgpu::BindGroupLayout,
}



pub struct RenderAssets {
	pub materials_storage: MaterialsStorage,
	pub test_model: ModelRenderData,
	pub depth: DepthRenderData,
	pub camera: CameraRenderData,
}

pub struct MaterialsStorage {
	pub list: Vec<MaterialRenderData>,
}

impl MaterialsStorage {
	pub fn new() -> Self {
		Self {
			list: vec!(),
		}
	}
}

pub struct MaterialRenderData {
	pub name: String,
	//pub texture: wgpu::Texture,
	//pub view: wgpu::TextureView,
	//pub sampler: wgpu::Sampler,
	pub bind_group: wgpu::BindGroup,
}

pub struct ModelRenderData {
	pub instances_buffer: wgpu::Buffer,
	pub instances_count: u32,
	pub meshes: Vec<MeshRenderData>,
}

pub struct MeshRenderData {
	pub vertex_buffer: wgpu::Buffer,
	pub index_buffer: wgpu::Buffer,
	pub index_count: u32,
	pub material_index: usize,
}

pub struct DepthRenderData {
	//pub texture: wgpu::Texture,
	pub view: wgpu::TextureView,
	//pub sampler: wgpu::Sampler,
}

pub struct CameraRenderData {
	pub buffer: wgpu::Buffer,
	pub bind_layout: wgpu::BindGroupLayout,
	pub bind_group: wgpu::BindGroup,
}



pub struct RenderPipelines {
	pub test: wgpu::RenderPipeline,
	pub skybox: wgpu::RenderPipeline,
}





pub struct TextureData {
	pub texture: wgpu::Texture,
	pub view: wgpu::TextureView,
	pub sampler: wgpu::Sampler,
	pub bind_group: wgpu::BindGroup
}



#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GenericVertex {
	pub position: [f32; 3],
	pub tex_coords: [f32; 2],
	pub normal: [f32; 3],
}

impl GenericVertex {
	pub const ATTRIBUTES: [wgpu::VertexAttribute; 3] = wgpu::vertex_attr_array![
		0 => Float32x3,
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



pub struct Instance {
	pub position: cgmath::Vector3<f32>,
	pub rotation: cgmath::Quaternion<f32>,
}

impl Instance {
	pub fn to_raw(&self) -> InstanceRaw {
		let model_data = cgmath::Matrix4::from_translation(self.position) * cgmath::Matrix4::from(self.rotation);
		InstanceRaw {
			model: model_data.into(),
		}
	}
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRaw {
	pub model: [[f32; 4]; 4],
}

impl InstanceRaw {
	pub const ATTRIBUTES: [wgpu::VertexAttribute; 4] = wgpu::vertex_attr_array![
		5 => Float32x4,
		6 => Float32x4,
		7 => Float32x4,
		8 => Float32x4
	];
	pub fn get_layout() -> wgpu::VertexBufferLayout<'static> {
		use std::mem;
		wgpu::VertexBufferLayout {
			array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
			step_mode: wgpu::VertexStepMode::Instance,
			attributes: &Self::ATTRIBUTES,
		}
	}
}



pub const VERTICES: &[GenericVertex] = &[
	GenericVertex { position: [-0.0868241, 0.49240386, 0.0], tex_coords: [0.4131759, 0.00759614], normal: [0.0, 0.0, 0.0] },
	GenericVertex { position: [-0.49513406, 0.06958647, 0.0], tex_coords: [0.0048659444, 0.43041354], normal: [0.0, 0.0, 0.0] },
	GenericVertex { position: [-0.21918549, -0.44939706, 0.0], tex_coords: [0.28081453, 0.949397], normal: [0.0, 0.0, 0.0] },
	GenericVertex { position: [0.35966998, -0.3473291, 0.0], tex_coords: [0.85967, 0.84732914], normal: [0.0, 0.0, 0.0] },
	GenericVertex { position: [0.44147372, 0.2347359, 0.0], tex_coords: [0.9414737, 0.2652641], normal: [0.0, 0.0, 0.0] },
];

pub const INDICES: &[u16] = &[
	0, 1, 4,
	1, 2, 4,
	2, 3, 4,
];





pub struct Camera {
	pub eye: cgmath::Point3<f32>,
	pub target: cgmath::Point3<f32>,
	pub up: cgmath::Vector3<f32>,
	pub fov: f32,
	pub near: f32,
	pub far: f32,
}

impl Camera {
	pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
		1.0, 0.0, 0.0, 0.0,
		0.0, 1.0, 0.0, 0.0,
		0.0, 0.0, 0.5, 0.5,
		0.0, 0.0, 0.0, 1.0,
	);	
	pub fn build_view_projection_matrix(&self, aspect_ratio: f32) -> cgmath::Matrix4<f32> {
		let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
		let proj = cgmath::perspective(cgmath::Deg (self.fov), aspect_ratio, self.near, self.far);
		Self::OPENGL_TO_WGPU_MATRIX * proj * view
	}
	pub fn default_data() -> [[f32; 4]; 4] {
		cgmath::Matrix4::identity().into()
	}
	pub fn new(pos: (f32, f32, f32)) -> Self {
		Self {
			eye: pos.into(),
			target: (0.0, 0.0, 0.0).into(),
			up: cgmath::Vector3::unit_y(),
			fov: 45.0,
			near: 0.1,
			far: 100.0,
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

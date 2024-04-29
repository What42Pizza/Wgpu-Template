use crate::prelude::*;
use cgmath::SquareMatrix;
use winit::keyboard::KeyCode;



pub struct ProgramData<'a> {
	
	pub window: &'a Window,
	pub pressed_keys: HashMap<KeyCode, bool>,
	pub frame_instant: Instant,
	
	pub render_context: wgpu_integration::RenderContextData<'a>,
	pub uniform_datas: UniformDatas,
	pub asset_datas: AssetDatas,
	pub world_datas: WorldDatas,
	pub render_pipelines: RenderPipelines,
	pub camera: Camera,
	
	pub start_time: Instant,
	pub fps_counter: FpsCounter,
	
}

impl<'a> ProgramData<'a> {
	pub fn key_is_down(&self, key: KeyCode) -> bool {
		self.pressed_keys.get(&key).cloned().unwrap_or(false)
	}
	pub fn step_dt(&mut self) -> f32 {
		let new_frame_instant = Instant::now();
		let dt = (new_frame_instant - self.frame_instant).as_secs_f32();
		self.frame_instant = new_frame_instant;
		dt
	}
}



pub struct UniformDatas {
	pub camera_binding: wgpu_integration::GeneralBindData,
}

pub struct AssetDatas {
	pub happy_tree_binding: wgpu_integration::TextureBindData,
}

pub struct WorldDatas {
	pub main_vertices: wgpu::Buffer,
	pub main_indices: wgpu::Buffer,
	pub main_index_count: u32,
}



pub struct RenderPipelines {
	pub main: wgpu::RenderPipeline,
}



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



pub const VERTICES: &[wgpu_integration::Vertex] = &[
	wgpu_integration::Vertex { position: [-0.0868241, 0.49240386, 0.0], tex_coords: [0.4131759, 0.00759614], },
	wgpu_integration::Vertex { position: [-0.49513406, 0.06958647, 0.0], tex_coords: [0.0048659444, 0.43041354], },
	wgpu_integration::Vertex { position: [-0.21918549, -0.44939706, 0.0], tex_coords: [0.28081453, 0.949397], },
	wgpu_integration::Vertex { position: [0.35966998, -0.3473291, 0.0], tex_coords: [0.85967, 0.84732914], },
	wgpu_integration::Vertex { position: [0.44147372, 0.2347359, 0.0], tex_coords: [0.9414737, 0.2652641], },
];

pub const INDICES: &[u16] = &[
	0, 1, 4,
	1, 2, 4,
	2, 3, 4,
];

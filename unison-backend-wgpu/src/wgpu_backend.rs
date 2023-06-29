use crate::*;
use unison_backend::*;
use wgpu::util::DeviceExt;

use std::collections::HashMap;

pub struct WgpuBackend {
	pub instance: wgpu::Instance,
	pub adapter: wgpu::Adapter,
	pub device: wgpu::Device,
	pub queue: wgpu::Queue,

	pub image_cache: HashMap<TextureId, wgpu::Texture>,
}

impl WgpuBackend {
	pub fn new() -> Self {
		let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
			backends: wgpu::Backends::PRIMARY,
			..Default::default()
		});

		let adapter = instance.enumerate_adapters(wgpu::Backends::all())
			//.filter(|adapter| adapter.is_surface_supported(&surface))
			.next()
			.unwrap(); // TODO

		let (device, queue) = pollster::block_on(adapter.request_device(
			&wgpu::DeviceDescriptor {
				features: wgpu::Features::TEXTURE_BINDING_ARRAY | wgpu::Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING,
				..Default::default()
			},
			None
		)).unwrap(); // TODO

		Self {
			instance,
			adapter,
			device,
			queue,

			image_cache: HashMap::new()
		}
	}
}

impl Default for WgpuBackend {
	fn default() -> Self {
		Self::new()
	}
}

impl Backend for WgpuBackend {
	type View<'a> = WgpuView<'a> where Self: 'a;
	type Surface = WgpuSurface;

	fn create_surface(&self, window: &winit::window::Window) -> Self::Surface {
		let size = window.inner_size();
		WgpuSurface::new(self, unsafe { self.instance.create_surface(window) }.unwrap(), (size.width, size.height))
	}

	fn create_view<'a>(&'a mut self, surface: &'a mut Self::Surface) -> Self::View<'a> {
		WgpuView::new(self, surface)
	}

	fn upload_texture(&mut self, tex: &Texture) -> TextureId {
		static TEX_ID: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

		let desc = wgpu::TextureDescriptor {
			label: None,
			size: wgpu::Extent3d { width: tex.width(), height: tex.height(), depth_or_array_layers: 1 },
			mip_level_count: 1,
			sample_count: 1,
			dimension: wgpu::TextureDimension::D2,
			format: texture_format_to_wgpu(tex.format()),
			usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
			view_formats: &[],
		};
		let wgpu_tex = self.device.create_texture_with_data(&self.queue, &desc, tex.as_bytes());

		let id = TextureId::new(TEX_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed));

		self.image_cache.insert(id, wgpu_tex);

		id
	}
}

fn texture_format_to_wgpu(format: TextureFormat) -> wgpu::TextureFormat {
	match format {
		TextureFormat::Rgba32F => wgpu::TextureFormat::Rgba32Float,
	}
}

pub struct WgpuSurface {
	surface: wgpu::Surface,
	tex: Option<wgpu::SurfaceTexture>,
	view: Option<wgpu::TextureView>,
	pipeline: QuadPipeline,
	window_size: (u32, u32),
}

impl WgpuSurface {
	pub fn create_surface_config(surface_caps: wgpu::SurfaceCapabilities, window_size: (u32, u32)) -> wgpu::SurfaceConfiguration {
		wgpu::SurfaceConfiguration {
			usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
			format: surface_caps.formats[0],
			width: window_size.0,
			height: window_size.1,
			present_mode: surface_caps.present_modes[0],
			alpha_mode: surface_caps.alpha_modes[0],
			view_formats: vec![]
		}
	}

	pub fn new(bcknd: &WgpuBackend, surface: wgpu::Surface, window_size: (u32, u32)) -> Self {
		let surface_caps = surface.get_capabilities(&bcknd.adapter);
		let surface_config = Self::create_surface_config(surface_caps, window_size);

		let pipeline = QuadPipeline::new(bcknd, &surface_config, window_size);

		Self {
			surface,
			tex: None,
			view: None,
			pipeline,
			window_size,
		}
	}

	pub fn reconfigure(&mut self, bcknd: &WgpuBackend, window_size: (u32, u32)) {
		self.window_size = window_size;

		let surface_caps = self.surface.get_capabilities(&bcknd.adapter);
		let surface_config = Self::create_surface_config(surface_caps, window_size);
		self.surface.configure(&bcknd.device, &surface_config);

		self.pipeline.reconfigure(bcknd, window_size);
	}

	pub fn ensure_surface_texture(&mut self) {
		if self.tex.is_none() {
			let tex = self.surface.get_current_texture().unwrap();
			
			let view = tex.texture.create_view(&wgpu::TextureViewDescriptor {
				..Default::default()
			});

			self.tex = Some(tex);
			self.view = Some(view);
		}
	}

	pub fn get_current_texture(&mut self) -> &wgpu::SurfaceTexture {
		self.ensure_surface_texture();
		self.tex.as_ref().unwrap() // TODO
	}
}

impl Surface<WgpuBackend> for WgpuSurface {
	fn reconfigure(&mut self, bcknd: &WgpuBackend, window_size: (u32, u32)) {
		self.reconfigure(bcknd, window_size);
	}
}


pub struct WgpuView<'a> {
	bcknd: &'a mut WgpuBackend,
	surface: &'a mut WgpuSurface,
	window_size: (u32, u32),
	state: smallvec::SmallVec<[WgpuViewState; 8]>,
}

impl<'a> WgpuView<'a> {
	pub fn new(bcknd: &'a mut WgpuBackend, surface: &'a mut WgpuSurface) -> Self {
		surface.ensure_surface_texture();
		surface.pipeline.set_clear(Color(0.0, 0.0, 0.0, 1.0));

		let window_size = surface.window_size;

		let mut state = smallvec::SmallVec::new();
		state.push(WgpuViewState::new(window_size));

		Self {
			bcknd,
			surface,
			window_size,
			state,
		}
	}

	pub fn get_state(&self) -> &WgpuViewState {
		self.state.last().unwrap() // state is never empty
	}

	pub fn get_state_mut(&mut self) -> &mut WgpuViewState {
		self.state.last_mut().unwrap() // state is never empty
	}
}

impl<'a> View for WgpuView<'a> {
	type B = WgpuBackend;

	fn push(&mut self) {
		self.state.push(self.state.last().unwrap().clone())
	}

	fn restore(&mut self) {
		self.state.pop();

		if self.state.len() == 0 {
			self.state.push(WgpuViewState::new(self.window_size))
		}
	}

	fn reset_viewport(&mut self) {
		*self.get_state_mut() = WgpuViewState::new(self.window_size);
	}

	fn viewport_size(&self) -> (u32, u32) {
		self.get_state().size
	}

	fn set_viewport_horizontal(&mut self, offset: u32, width: u32) {
		let state = self.get_state_mut();
		state.pos.0 += offset;
		state.size.0 = width;
	}

	fn set_viewport_vertical(&mut self, offset: u32, height: u32) {
		let state = self.get_state_mut();
		state.pos.1 += offset;
		state.size.1 = height;
	}

	fn apply_bounds(&mut self, bounds: Bounds) {
		let state = self.get_state_mut();

		state.pos.0 += bounds.left;
		state.size.0 -= bounds.left + bounds.right;

		state.pos.1 += bounds.top;
		state.size.1 -= bounds.top + bounds.bottom;
	}

	fn fill(&mut self, finish: Finish) {
		let state = self.get_state();
		let color = match finish {
			Finish::Color(c) => c,
			_ => todo!() // TODO
		};
		self.surface.pipeline.queue_quad(self.bcknd, (state.pos.0 as i32, state.pos.1 as i32), state.size, color, None, None, self.surface.view.as_ref().unwrap()).unwrap()
	}

	fn draw_rect(&mut self, pos: (i32, i32), size: (u32, u32), color: Color, tex: Option<TextureId>, tex_offset: Option<(u32, u32)>) {
		let state = self.get_state();

		let pos = (state.pos.0 as i32 + pos.0 as i32, state.pos.1 as i32 + pos.1 as i32);

		let tex_coords = if let Some(tex_offset) = tex_offset {
			let from_x = tex_offset.0 as f32 / 1024.0;
			let from_y = tex_offset.1 as f32 / 1024.0;
			let to_x = (tex_offset.0 + size.0) as f32 / 1024.0;
			let to_y = (tex_offset.1 + size.1) as f32 / 1024.0;
	
			Some(([from_x, from_y], [from_x, to_y], [to_x, to_y], [to_x, from_y]))
		} else {
			None
		};

		self.surface.pipeline.queue_quad(
			self.bcknd,
			pos,
			size,
			color,
			tex,
			tex_coords,
			self.surface.view.as_ref().unwrap()
		).unwrap()

	}

	fn submit(self) {
		self.surface.pipeline.flush(self.surface.view.as_ref().unwrap(), self.bcknd).unwrap();
		self.surface.tex.take().map(|t| t.present());
		self.surface.view.take();
	}

	fn backend(&mut self) -> &mut Self::B {
		self.bcknd
	}
}


#[derive(Debug, Clone, Copy)]
pub struct WgpuViewState {
	pos: (u32, u32),
	size: (u32, u32),
}

impl WgpuViewState {
	pub fn new(window_size: (u32, u32)) -> Self {
		Self {
			pos: (0, 0),
			size: window_size,
		}
	}
}



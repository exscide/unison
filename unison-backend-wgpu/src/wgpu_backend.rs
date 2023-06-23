use crate::*;
use unison_backend::*;

pub struct WgpuBackend {
	pub instance: wgpu::Instance,
	pub adapter: wgpu::Adapter,
	pub device: wgpu::Device,
	pub queue: wgpu::Queue,
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
			queue
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

	fn create_view<'a>(&'a self, surface: &'a mut Self::Surface) -> Self::View<'a> {
		WgpuView::new(self, surface)
	}

	fn submit_view<'a>(&'a self, view: Self::View<'a>) {
		view.surface.pipeline.flush(view.surface.view.as_ref().unwrap(), view.bcknd).unwrap();
		view.surface.tex.take().map(|t| t.present());
		view.surface.view.take();
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
	bcknd: &'a WgpuBackend,
	surface: &'a mut WgpuSurface,
	window_size: (u32, u32),
	state: Vec<WgpuViewState>,
}

impl<'a> WgpuView<'a> {
	pub fn new(bcknd: &'a WgpuBackend, surface: &'a mut WgpuSurface) -> Self {
		surface.ensure_surface_texture();
		surface.pipeline.set_clear(Color(0.0, 0.0, 0.0, 1.0));

		let window_size = surface.window_size;

		let mut state = Vec::with_capacity(10);
		state.push(WgpuViewState::new(window_size));

		Self {
			bcknd,
			surface,
			window_size,
			state,
		}
	}

	pub fn get_state(&self) -> &WgpuViewState {
		self.state.last().unwrap()
	}

	pub fn get_state_mut(&mut self) -> &mut WgpuViewState {
		self.state.last_mut().unwrap()
	}
}

impl<'a> View for WgpuView<'a> {
	fn push(&mut self) {
		self.state.push(self.state.last().unwrap().clone())
	}

	fn restore(&mut self) {
		self.state.pop();

		if self.state.len() == 0 {
			self.state.push(WgpuViewState::new(self.window_size))
		}
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

	fn fill(&mut self, color: Color) {
		let state = self.get_state();
		self.surface.pipeline.queue_quad(self.bcknd, state.pos, state.size, color, self.surface.view.as_ref().unwrap()).unwrap()
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



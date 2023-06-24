use crate::*;

use std::collections::HashMap;

use winit::event_loop::{ EventLoop, EventLoopWindowTarget };
use winit::window::{ WindowId, Window };
use winit::event::{ Event, WindowEvent };


pub struct App<B: Backend + 'static = unison_backend_wgpu::WgpuBackend> {
	viewports: HashMap<WindowId, Viewport<B>>,
	window_queue: Vec<Box<dyn DynPage<B>>>,
	backend: B,
	font_state: FontState,
}

impl App<unison_backend_wgpu::WgpuBackend> {
	pub fn new() -> Self {
		Self {
			viewports: HashMap::new(),
			window_queue: Vec::with_capacity(1),
			backend: unison_backend_wgpu::WgpuBackend::new(),
			font_state: FontState::new(),
		}
	}
}

impl<B: Backend + 'static> App<B> {
	pub fn with_backend(backend: B) -> Self {
		Self {
			viewports: HashMap::new(),
			window_queue: Vec::with_capacity(1),
			backend,
			font_state: FontState::new(),
		}
	}

	pub fn with_window<T: Component + 'static>(mut self, page: Page<T>) -> Self {
		self.window_queue.push(Box::new(page));
		self
	}

	fn handle_window_event(&mut self, id: WindowId, ev: WindowEvent) {
		let vp = match self.viewports.get_mut(&id) {
			Some(v) => v,
			None => return,
		};

		match ev {
			WindowEvent::Resized(size) => {
				vp.reconfigure(&self.backend, (size.width.max(1), size.height.max(1)));
			},
			WindowEvent::Moved(p) => {
				vp.page.emit_window_moved((p.x, p.y));
			},
			WindowEvent::Focused(f) => {
				vp.page.emit_window_focus_changed(f);
			}
			_ => {}
		}

		if vp.page.take_redraw_request() {
			vp.get_window().request_redraw();
		}
	}

	fn handle_redraw(&mut self, id: WindowId) {
		if let Some(v) = self.viewports.get_mut(&id) {
			v.draw(&mut self.backend, &mut self.font_state);
		}
	}

	pub fn run(mut self) -> ! {
		let ev_loop = EventLoop::new();

		let font = self.font_state.find_font(Attrs::new(), 16.0);
		self.font_state.upload_font(font, &mut self.backend);

		self.viewports = self.window_queue.drain(..)
			.map(|page| Viewport::new(&ev_loop, &self.backend, page).unwrap()) // TODO: get rid of unwrap
			.collect();

		ev_loop.run(move |ev, _, _cf| {
			match ev {
				Event::WindowEvent { window_id, event } => self.handle_window_event(window_id, event),
				Event::RedrawRequested(id) => self.handle_redraw(id),
				_ => {}
			}
		});
	}
}

struct Viewport<B: Backend> {
	window: Window,
	surface: B::Surface,
	pub(crate) page: Box<dyn DynPage<B>>,
}

impl<B: Backend> Viewport<B> {
	pub fn new<T: 'static>(ev_loop: &EventLoopWindowTarget<T>, bcknd: &B, page: Box<dyn DynPage<B>>) -> Result<(WindowId, Self), winit::error::OsError> {
		let mut window = winit::window::WindowBuilder::new()
			.with_title("")
			.build(ev_loop)?;

		let surface = bcknd.create_surface(&window);

		page.update_window(&mut window);

		Ok((window.id(), Viewport {
			window,
			surface,
			page,
		}))
	}

	pub fn get_window(&self) -> &winit::window::Window {
		&self.window
	}

	pub fn reconfigure(&mut self, bcknd: &B, window_size: (u32, u32)) {
		self.surface.reconfigure(bcknd, window_size);
	}

	pub fn draw(&mut self, bcknd: &mut B, font_state: &mut FontState) {
		self.page.draw(&mut self.surface, bcknd, font_state);
	}
}

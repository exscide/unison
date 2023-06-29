use crate::*;
use crate::container::ComponentTree;


pub struct Page<T: Component> {
	tree: ComponentTree<T>,
	title: Option<String>,
	state: State,
}

impl<T: Component> Page<T> {
	pub fn new(root: T) -> Self {
		let mut state = State::new();
		let tree = ComponentTree::new(root, &mut state);

		Self {
			tree,
			title: None,
			state,
		}
	}

	pub fn with_title(&mut self, title: &str) -> &mut Self {
		self.title = Some(String::from(title));
		self
	}

	pub fn draw<B: Backend>(&self, surface: &mut B::Surface, bcknd: &mut B, font_state: &mut FontState) {
		let mut view = bcknd.create_view(surface);
		self.tree.draw::<B>(&self.state, &mut view, font_state);
		view.submit();
	}

	pub fn update_window(&self, win: &mut winit::window::Window) {
		if let Some(title) = &self.title {
			win.set_title(title);
		}
	}
}

impl<T: Component, B: Backend> DynPage<B> for Page<T> {
	fn draw(&self, surface: &mut B::Surface, bcknd: &mut B, font_state: &mut FontState) {
		self.draw::<B>(surface, bcknd, font_state)
	}

	fn update_window(&self, win: &mut winit::window::Window) {
		self.update_window(win)
	}

	fn take_redraw_request(&mut self) -> bool {
		let r = self.state.request_redraw;
		self.state.request_redraw = false;
		r
	}

	fn emit_window_moved(&mut self, pos: (i32, i32)) {
		self.state.set(self.state.window_pos, pos);
	}

	fn emit_window_focus_changed(&mut self, focused: bool) {
		self.state.set(self.state.window_focused, focused);
	}
}

pub(crate) trait DynPage<B: Backend> {
	fn draw(&self, surface: &mut B::Surface, bcknd: &mut B, font_state: &mut FontState);
	fn update_window(&self, win: &mut winit::window::Window);

	fn take_redraw_request(&mut self) -> bool;

	fn emit_window_moved(&mut self, pos: (i32, i32));
	fn emit_window_focus_changed(&mut self, focused: bool);
}

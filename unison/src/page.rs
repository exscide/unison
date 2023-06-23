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

	pub fn draw<B: Backend>(&self, surface: &mut B::Surface, bcknd: &mut B) {
		let mut view = bcknd.create_view(surface);
		self.tree.draw::<B>(&self.state, &mut view);
		bcknd.submit_view(view);
	}

	pub fn update_window(&self, win: &mut winit::window::Window) {
		if let Some(title) = &self.title {
			win.set_title(title);
		}
	}
}

impl<T: Component, B: Backend> DynPage<B> for Page<T> {
	fn draw(&self, surface: &mut B::Surface, bcknd: &mut B) {
		self.draw::<B>(surface, bcknd)
	}

	fn update_window(&self, win: &mut winit::window::Window) {
		self.update_window(win)
	}
}

pub(crate) trait DynPage<B: Backend> {
	fn draw(&self, surface: &mut B::Surface, bcknd: &mut B);
	fn update_window(&self, win: &mut winit::window::Window);
}

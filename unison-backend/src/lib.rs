pub mod types;
use types::*;

pub trait Backend: Default {
	type View<'a>: View where Self: 'a;
	type Surface: Surface<Self> + 'static;

	fn create_surface(&self, window: &winit::window::Window) -> Self::Surface;
	fn create_view<'a>(&'a self, surface: &'a mut Self::Surface) -> Self::View<'a>;
	fn submit_view<'a>(&'a self, view: Self::View<'a>);
}

pub trait View {
	/// Make a copy of the current state and push it onto the stack.
	fn push(&mut self);
	/// Restore the previous state.
	fn restore(&mut self);

	fn viewport_size(&self) -> (u32, u32);

	fn set_viewport_horizontal(&mut self, offset: u32, width: u32);

	fn set_viewport_vertical(&mut self, offset: u32, width: u32);


	/// Apply some [Bounds] to the current viewport.
	fn apply_bounds(&mut self, bounds: Bounds);

	fn fill(&mut self, color: Color);
}

pub trait Surface<B: Backend> {
	fn reconfigure(&mut self, bcknd: &B, window_size: (u32, u32));
}

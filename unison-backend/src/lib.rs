pub mod types;
use types::*;

pub trait Backend: Default {
	type View<'a>: View<B = Self> where Self: 'a;
	type Surface: Surface<Self> + 'static;

	fn create_surface(&self, window: &winit::window::Window) -> Self::Surface;
	fn create_view<'a>(&'a mut self, surface: &'a mut Self::Surface) -> Self::View<'a>;

	fn upload_texture(&mut self, tex: &Texture) -> TextureId;
}

pub trait View {
	type B: Backend;

	/// Make a copy of the current state and push it onto the stack.
	fn push(&mut self);
	/// Restore the previous state.
	fn restore(&mut self);


	/// Reset the current viewport to fit the whole screen again.
	fn reset_viewport(&mut self);

	/// Get the current viewports size.
	fn viewport_size(&self) -> (u32, u32);

	fn set_viewport_horizontal(&mut self, offset: u32, width: u32);

	fn set_viewport_vertical(&mut self, offset: u32, width: u32);

	/// Apply some [Bounds] to the current viewport.
	fn apply_bounds(&mut self, bounds: Bounds);

	/// Fill the current viewport with a [Finish].
	fn fill(&mut self, finish: Finish);

	fn draw_rect(&mut self, pos: (i32, i32), size: (u32, u32), color: Color, tex: Option<TextureId>, tex_offset: Option<(u32, u32)>);

	fn submit(self);
	fn backend(&mut self) -> &mut Self::B;
}

pub trait Surface<B: Backend> {
	fn reconfigure(&mut self, bcknd: &B, window_size: (u32, u32));
}

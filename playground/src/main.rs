use unison::*;


struct A;

impl Component for A {
	type Child = ();

	fn build(&self, _state: &mut State) -> Self::Child {}

	fn draw<'a, B: Backend>(&self, _state: &State, view: &mut B::View<'a>) {
		view.fill(Color(1.0, 1.0, 0.0, 1.0));
	}
}

struct B;

impl Component for B {
	type Child = ();

	fn build(&self, _state: &mut State) -> Self::Child {}

	fn draw<'a, B: Backend>(&self, _state: &State, view: &mut B::View<'a>) {
		view.fill(Color(0.0, 1.0, 1.0, 1.0));
	}

	fn layout(&self, _state: &mut State) -> Layout {
		let mut layout = Layout::new();
		layout.set_flex(2);
		layout
	}
}

struct Yote;
impl Component for Yote {
	type Child = (A, B);

	fn build(&self, _state: &mut State) -> Self::Child {
		(A, B)
	}

	fn layout(&self, _state: &mut State) -> Layout {
		let mut layout = Layout::new();

		layout.set_flex(3);

		layout.set_stack_spacing(10);

		layout
	}
}


struct C;

impl Component for C {
	type Child = ();

	fn build(&self, _state: &mut State) -> Self::Child {}

	fn draw<'a, B: Backend>(&self, _state: &State, view: &mut B::View<'a>) {
		view.fill(Color(1.0, 1.0, 1.0, 1.0));
	}

	fn layout(&self, _state: &mut State) -> Layout {
		let mut layout = Layout::new();
		layout.set_flex(2);
		layout
	}
}


struct MainView;

impl Component for MainView {
	type Child = (Yote, C);

	fn build(&self, _state: &mut State) -> Self::Child {
		(Yote, C)
	}

	fn draw<'a, B: Backend>(&self, _state: &State, view: &mut B::View<'a>) {
		view.fill(Color(1.0, 0.0, 1.0, 1.0));
	}

	fn layout(&self, state: &mut State) -> Layout {
		let focused = state.window_focused;
		state.redraw_on_change(focused);

		let mut layout = Layout::new();

		layout.set_stack_orientation(bind!(focused => if focused { Orientation::Horizontal } else { Orientation::Vertical }));
		layout.set_stack_spacing(10);

		layout.set_padding(Bounds::new(10, 10, 10, 10));

		layout
	}
}


fn main() {
	let app = App::new()
		.with_window(Page::new(MainView));
	app.run();

	// let app = SimpleApp::new(MainView {});
	// app.run();
}

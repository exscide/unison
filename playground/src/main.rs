use unison::*;


struct A;

impl Component for A {
	type Child = ();

	fn build(&self, _state: &mut State) -> Self::Child {}

	fn draw<'a, B: Backend>(&self, _state: &State, view: &mut B::View<'a>, font_state: &mut FontState) {
		view.fill(Color(1.0, 1.0, 0.0, 1.0).into());
	}
}


struct B;

impl Component for B {
	type Child = ();

	fn build(&self, _state: &mut State) -> Self::Child {}

	fn draw<'a, B: Backend>(&self, _state: &State, view: &mut B::View<'a>, font_state: &mut FontState) {
		view.fill(Color(0.0, 1.0, 1.0, 1.0).into());
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
	type Child = Label;

	fn build(&self, _state: &mut State) -> Self::Child {
		Label { text: "On it differed repeated wandered required in. Then girl neat why yet knew rose spot. Moreover property we he kindness greatest be oh striking laughter. In me he at collecting affronting principles apartments. Has visitor law attacks pretend you calling own excited painted. Contented attending smallness it oh ye unwilling. Turned favour man two but lovers. Suffer should if waited common person little oh. Improved civility graceful sex few smallest screened settling. Likely active her warmly has. ❤️".to_owned() }
	}

	fn draw<'a, B: Backend>(&self, _state: &State, view: &mut B::View<'a>, font_state: &mut FontState) {
		view.fill(Color(1.0, 1.0, 1.0, 1.0).into());
	}

	fn layout(&self, _state: &mut State) -> Layout {
		let mut layout = Layout::new();
		layout.set_flex(2);
		layout.set_padding(Bounds::new(5, 5, 5, 5));
		layout
	}
}


struct MainView;

impl Component for MainView {
	type Child = (Yote, C);

	fn build(&self, _state: &mut State) -> Self::Child {
		(Yote, C)
	}

	fn draw<'a, B: Backend>(&self, _state: &State, view: &mut B::View<'a>, font_state: &mut FontState) {
		view.fill(Color(1.0, 0.0, 1.0, 1.0).into());
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
	// let mut f = FontState::new();
	// f.find_font(Attrs::new(), 18.0);

	let app = App::new()
		.with_window(Page::new(MainView));
	app.run();

	// let app = SimpleApp::new(MainView {});
	// app.run();
}

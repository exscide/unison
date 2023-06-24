use crate::*;


pub struct Label {
	pub text: String,
}

impl Component for Label {
	type Child = ();

	fn build(&self, _: &mut State) -> Self::Child {
		()
	}

	fn draw<'a, B: Backend>(&self, _: &State, view: &mut B::View<'a>, font_state: &mut FontState) {
		let mut buf = cosmic_text::Buffer::new(&mut font_state.font_system, cosmic_text::Metrics { font_size: 16.0, line_height: 16.0 });

		{
			let mut buf = buf.borrow_with(&mut font_state.font_system);

			let s = view.viewport_size();
			buf.set_size(s.0 as f32, s.1 as f32);
			
			// Add some text!
			buf.set_text(&self.text, Attrs::new());
			
			// Perform shaping as desired
			buf.shape_until_scroll();
		}

		for run in buf.layout_runs() {
			for glyph in run.glyphs.iter() {
				let id = glyph.cache_key.glyph_id;
				let font = &font_state.fonts[0];
				if let Some((g, tex_id)) = font.get_glyph(id) {

					view.draw_rect((glyph.x as i32 + g.left, glyph.y_int as i32 - g.top + 16), (g.width, g.height), Color(0.0, 0.0, 0.0, 1.0), tex_id, (g.offset_x, g.offset_y))
				}
			}
		}
	}
}

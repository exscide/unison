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

			buf.set_text(&self.text, Attrs::new().family(cosmic_text::Family::Name("Times New Roman")));

			buf.shape_until_scroll();
		}

		for line in buf.layout_runs() {
			let line_y = line.line_y as i32;

			for glyph in line.glyphs.iter() {
				let glyph_id = glyph.cache_key.glyph_id;

				let fid = font_state.ensure_font(
					glyph.cache_key.font_id,
					unsafe { std::mem::transmute(glyph.cache_key.font_size_bits) }, view.backend());
				let font = font_state.get_font::<B>(fid);

				if let Some((g, tex_id)) = font.get_glyph(glyph_id) {
					// view.draw_rect(
					// 	(glyph.x_int + g.left, line_y + glyph.y_int as i32 - g.top),
					// 	(g.width, g.height),
					// 	Color(1.0, 0.0, 1.0, 0.2),
					// 	None,
					// 	None
					// );

					view.draw_rect(
						(glyph.x_int + g.left, line_y + glyph.y_int as i32 - g.top),
						(g.width, g.height),
						Color(0.0, 0.0, 0.0, 1.0),
						Some(tex_id),
						Some((g.offset_x, g.offset_y))
					)
				}
			}
		}
	}
}

use image::{buffer::ConvertBuffer, ImageBuffer};

use crate::*;

use std::sync::Arc;
use std::collections::HashMap;


pub struct FontState {
	pub font_system: cosmic_text::FontSystem,
	pub swash_cache: cosmic_text::SwashCache,
	pub fonts: Vec<Font>,
}

impl FontState {
	pub fn new() -> Self {
		Self {
			font_system: cosmic_text::FontSystem::new(),
			swash_cache: cosmic_text::SwashCache::new(),
			fonts: Vec::new(),
		}
	}

	pub fn find_font(&mut self, attrs: cosmic_text::Attrs, size: f32) -> FontId {
		let m = self.font_system.get_font_matches(attrs);

		let id = *m.first().unwrap(); // TODO
		let font = self.font_system.get_font(id).unwrap();

		// TODO
		for (i, f) in self.fonts.iter().enumerate() {
			if Arc::ptr_eq(&f.font, &font) && f.size == size {
				return FontId(i);
			}
		}

		let mut f = Font::new(font, size);

		f.cache(self);

		self.fonts.push(f);

		FontId(self.fonts.len() - 1)
	}

	pub fn upload_font<B: Backend>(&mut self, id: FontId, bcknd: &mut B) {
		let font = &mut self.fonts[id.0];
		for page in &mut font.pages {
			if page.tex_id.is_none() {
				page.tex_id = Some(bcknd.upload_texture(&page.tex));
			}
		}
	}
}

pub struct Font {
	font: Arc<cosmic_text::Font>,
	size: f32,
	pages: Vec<CachePage>,
	glyphs: HashMap<u16, Glyph>,
}

impl Font {
	pub fn new(font: Arc<cosmic_text::Font>, size: f32) -> Self {
		Self {
			font,
			size,
			pages: Vec::new(),
			glyphs: HashMap::new()
		}
	}

	pub fn cache(&mut self, state: &mut FontState) {
		self.pages.clear();
		self.pages.push(CachePage::new());

		self.font.as_swash().charmap().enumerate(|_, id| {
			if let Some(img) = state.swash_cache.get_image(&mut state.font_system, cosmic_text::CacheKey::new(self.font.id(), id, self.size, (0.0, 0.0)).0).as_ref() {

				// this is literal lunacy and whoever designed that api should cease to exist immediately, for the greater good
				loop {
					let cp = self.pages.last_mut().unwrap();
					match cp.add_glyph(img) {
						None => {
							self.pages.push(CachePage::new());
						},
						Some(v) => {
							self.glyphs.insert(id, Glyph { page: self.pages.len() - 1, offset_x: v.0, offset_y: v.1, width: v.2, height: v.3, left: img.placement.left, top: img.placement.top });
							break;
						}
					}
				}

			}
		});
	}

	pub fn get_glyph(&self, id: u16) -> Option<(Glyph, TextureId)> {
		self.glyphs.get(&id)
			.map(|g| (*g, self.pages[g.page].tex_id.unwrap()))
	}
}

fn save(cp: &CachePage) {
	let mut buf = image::ImageBuffer::<image::Rgba<f32>, Vec<_>>::new(PAGE_SIZE, PAGE_SIZE);
	buf.copy_from_slice(bytemuck::cast_slice(cp.tex.as_bytes()));

	let buf: ImageBuffer<image::Rgba<u16>, Vec<_>> = buf.convert();

	buf.save(String::from("cp.png")).unwrap()
}

pub struct FontId(usize);


const PAGE_SIZE: u32 = 1024;

struct CachePage {
	tex: Texture,
	tex_id: Option<TextureId>,
	cur_y: u32,
	cur_x: u32,
	cur_max_glyph_height: u32,
}

impl CachePage {
	pub fn new() -> Self {
		let mut tex = Texture::new(PAGE_SIZE, PAGE_SIZE, TextureFormat::Rgba32F);

		tex.copy_from_slice(&[0u8; PAGE_SIZE as usize * PAGE_SIZE as usize * 16]);

		Self {
			tex,
			tex_id: None,
			cur_y: 0,
			cur_x: 0,
			cur_max_glyph_height: 0,
		}
	}

	fn copy_glyph(&mut self, glyph: &cosmic_text::SwashImage) {
		for glyph_y in 0..glyph.placement.height {
			for glyph_x in 0..glyph.placement.width {

				let tex_pos = ((self.cur_x + glyph_x) * 16 + (self.cur_y + glyph_y) * PAGE_SIZE * 16) as usize;
				let b = self.tex.as_bytes_mut();

				let glyph_pos = glyph_x + glyph_y * glyph.placement.width;
				let val = glyph.data[glyph_pos as usize] as f32 / 255.0;
				let val = val.to_ne_bytes();

				for channel in 0usize..4 {
					b[tex_pos+channel*4] = val[0];
					b[tex_pos+channel*4+1] = val[1];
					b[tex_pos+channel*4+2] = val[2];
					b[tex_pos+channel*4+3] = val[3];
				}
			}
		}
	}

	pub fn add_glyph(&mut self, glyph: &cosmic_text::SwashImage) -> Option<(u32, u32, u32, u32)> {
		if glyph.placement.width > PAGE_SIZE || glyph.placement.height > PAGE_SIZE {
			panic!()
		}

		if self.cur_x + glyph.placement.width > PAGE_SIZE {
			self.cur_y += self.cur_max_glyph_height;
			self.cur_x = 0;
		}

		if self.cur_y + glyph.placement.height > PAGE_SIZE {
			return None;
		}

		let bounds = (self.cur_x, self.cur_y, glyph.placement.width, glyph.placement.height);

		self.copy_glyph(glyph);

		self.cur_x += glyph.placement.width;

		if self.cur_max_glyph_height < glyph.placement.height {
			self.cur_max_glyph_height = glyph.placement.height;
		}

		Some(bounds)
	}
}

/// What area from what page to draw for a given glyph.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Glyph {
	pub page: usize,
	pub offset_x: u32,
	pub offset_y: u32,
	pub width: u32,
	pub height: u32,

	pub left: i32,
	pub top: i32,
}


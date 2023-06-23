
#[derive(Debug, Clone, Copy, Default)]
pub struct Bounds {
	pub top: u32,
	pub left: u32,
	pub bottom: u32,
	pub right: u32,
}

impl Bounds {
	pub fn new(top: u32, left: u32, bottom: u32, right: u32) -> Self {
		Self { top, left, bottom, right }
	}
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color(pub f64, pub f64, pub f64, pub f64);

impl Into<[f64; 4]> for Color {
	fn into(self) -> [f64; 4] {
		[self.0, self.1, self.2, self.3]
	}
}

impl Into<[f32; 4]> for Color {
	fn into(self) -> [f32; 4] {
		[self.0 as f32, self.1 as f32, self.2 as f32, self.3 as f32]
	}
}

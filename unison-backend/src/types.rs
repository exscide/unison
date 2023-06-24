

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


pub enum Finish {
	Color(Color),
	Texture(TextureId),
}

impl From<Color> for Finish {
	fn from(value: Color) -> Self {
		Self::Color(value)
	}
}

impl From<TextureId> for Finish {
	fn from(value: TextureId) -> Self {
		Self::Texture(value)
	}
}


#[derive(Debug, Clone)]
pub struct Texture {
	data: Vec<u8>,
	format: TextureFormat,
	width: u32,
	height: u32,
}

impl Texture {
	pub fn new(width: u32, height: u32, format: TextureFormat) -> Self {
		Self {
			data: Vec::with_capacity(width as usize * height as usize * format.pixel_size()),
			format: format,
			width,
			height,
		}
	}

	/// Get the width of the image in pixel.
	pub fn width(&self) -> u32 {
		self.width
	}

	/// Get the height of the image in pixel.
	pub fn height(&self) -> u32 {
		self.height
	}

	/// Get the format of the image.
	pub fn format(&self) -> TextureFormat {
		self.format
	}

	/// Get a slice of u8 to the image data.
	pub fn as_bytes(&self) -> &[u8] {
		&self.data
	}

	/// Get a mutable slice of u8 to the image data.
	pub fn as_bytes_mut(&mut self) -> &mut [u8] {
		&mut self.data
	}

	/// Overwrite the image data with the contents of a slice.
	pub fn copy_from_slice(&mut self, data: &[u8]) {
		self.data.clear();

		let dlen = data.len();
		let elen = self.width as usize * self.height as usize * self.format.pixel_size();

		if dlen != elen {
			panic!("invalid slice length: {}, expected: {}", dlen, elen); // TODO
		}

		self.data.extend_from_slice(data)
	}

	/// Get a [Texture] from an [image::DynamicImage].
	pub fn from_image(img: image::DynamicImage) -> Self {
		use image::EncodableLayout;

		let width = img.width();
		let height = img.height();

		let (data, format) = match img {
			image::DynamicImage::ImageRgb32F(buf) => (Vec::from(buf.as_bytes()), TextureFormat::Rgba32F),
			_ => (Vec::from(img.into_rgb32f().as_bytes()), TextureFormat::Rgba32F)
		};

		Self {
			data,
			format,
			width,
			height,
		}
	}

	/// Try to load image form a slice.
	pub fn from_bytes(data: &[u8]) -> image::ImageResult<Self> {
		Ok(Self::from_image(image::load_from_memory(data)?))
	}

	/// Try to load an image from a file.
	pub fn load<P: AsRef<std::path::Path>>(path: P) -> image::ImageResult<Self> {
		Ok(Self::from_image(image::open(path)?))
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureId(usize);

impl TextureId {
	pub fn new(id: usize) -> Self {
		Self(id)
	}

	pub fn id(&self) -> usize {
		self.0
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureFormat {
	/// Red, Green, Blue, Alpha of type f32
	Rgba32F,
}

impl TextureFormat {
	pub fn pixel_size(&self) -> usize {
		match self {
			Self::Rgba32F => 16,
		}
	}
}

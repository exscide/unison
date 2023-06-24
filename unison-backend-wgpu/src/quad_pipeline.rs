use crate::*;

use indexmap::IndexSet;


const TEXTURE_QUEUE_SIZE: u32 = 12;

pub struct QuadPipeline<
	const VC: usize = {10_000 * 4}, // vertex buffer size
	const IC: usize = {10_000 * 6}, // index buffer size
> {
	camera_buffer: wgpu::Buffer,
	uniform_bind_group: wgpu::BindGroup,

	pipeline: wgpu::RenderPipeline,
	vertex_buffer: wgpu::Buffer,
	index_buffer: wgpu::Buffer,
	vertices: Vec<QuadVertex>,
	indices: Vec<u32>,

	clear_queued: Option<Color>,

	texture_queue: IndexSet<TextureId>,

	texture_bind_group_layout: wgpu::BindGroupLayout,
	texture_bind_group: wgpu::BindGroup,
	fallback_texture_view: wgpu::TextureView,
	fallback_sampler: wgpu::Sampler,
}

impl<const VC: usize, const IC: usize> QuadPipeline<VC, IC> {

	fn create_fallback_tex(device: &wgpu::Device, queue: &wgpu::Queue) -> (wgpu::TextureView, wgpu::Sampler) {
		use wgpu::util::DeviceExt;

		let tex = device.create_texture_with_data(&queue, &wgpu::TextureDescriptor {
			label: Some("Quad Fallback Texture"),
			size: wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
			mip_level_count: 1,
			sample_count: 1,
			dimension: wgpu::TextureDimension::D2,
			format: wgpu::TextureFormat::Bgra8Unorm,
			usage: wgpu::TextureUsages::TEXTURE_BINDING,
			view_formats: &[],
		}, &[u8::MAX, u8::MAX, u8::MAX, u8::MAX]);

		let view = tex.create_view(&wgpu::TextureViewDescriptor::default());

		let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
			label: Some("Pipeline2d Fallback Sampler"),
			address_mode_u: wgpu::AddressMode::ClampToEdge,
			address_mode_v: wgpu::AddressMode::ClampToEdge,
			address_mode_w: wgpu::AddressMode::ClampToEdge,
			mag_filter: wgpu::FilterMode::Nearest,
			min_filter: wgpu::FilterMode::Nearest,
			mipmap_filter: wgpu::FilterMode::Nearest,
			..Default::default()
		});

		(view, sampler)
	}


	fn create_texture_bind_group(bcknd: &WgpuBackend, layout: &wgpu::BindGroupLayout, texture_queue: &IndexSet<TextureId>, fallback_texture_view: &wgpu::TextureView, fallback_sampler: &wgpu::Sampler) -> Result<wgpu::BindGroup> {

		let mut v = Vec::new();
		let mut s = Vec::new();

		let mut views: [&wgpu::TextureView; TEXTURE_QUEUE_SIZE as usize] = [fallback_texture_view; TEXTURE_QUEUE_SIZE as usize];
		let mut samplers: [&wgpu::Sampler; TEXTURE_QUEUE_SIZE as usize] = [fallback_sampler; TEXTURE_QUEUE_SIZE as usize];


		for (_, id) in texture_queue.iter().enumerate() {
			let tex = bcknd.image_cache.get(id).ok_or(())?;

			v.push(tex.create_view(&wgpu::TextureViewDescriptor {
				..Default::default()
			}));
			s.push(bcknd.device.create_sampler(&wgpu::SamplerDescriptor::default()));
		}

		for (i, _) in texture_queue.iter().enumerate() {

			views[i+1] = &v[i];
			samplers[i+1] = &s[i];
		}

		let bind_group = bcknd.device.create_bind_group(&wgpu::BindGroupDescriptor {
			label: Some("Pipeline2d BindGroup"),
			layout: layout, entries: &[
				wgpu::BindGroupEntry {
					binding: 0,
					resource: wgpu::BindingResource::TextureViewArray(&views),
				},
				wgpu::BindGroupEntry {
					binding: 1,
					resource: wgpu::BindingResource::SamplerArray(&samplers),
				},
			]
		});

		Ok(bind_group)
	}

	fn create_uniform_bind_group(device: &wgpu::Device, camera_buffer: &wgpu::Buffer) -> (wgpu::BindGroupLayout, wgpu::BindGroup) {
		let uniform_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
			label: Some("Pipeline2d BindGroupLayout"),
			entries: &[
				// camera
				wgpu::BindGroupLayoutEntry {
					binding: 0,
					visibility: wgpu::ShaderStages::VERTEX,
					ty: wgpu::BindingType::Buffer {
						ty: wgpu::BufferBindingType::Uniform,
						has_dynamic_offset: false,
						min_binding_size: None
					},
					count: None,
				},
			]
		});

		let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
			label: Some("Pipeline2d Uniform BindGroup"),
			layout: &uniform_bind_group_layout,
			entries: &[
				wgpu::BindGroupEntry {
					binding: 0,
					resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
						buffer: &camera_buffer,
						offset: 0,
						size: None,
					}),
				}
			]
		});

		(uniform_bind_group_layout, uniform_bind_group)
	}

	pub fn new(bcknd: &WgpuBackend, surface_config: &wgpu::SurfaceConfiguration, window_size: (u32, u32)) -> Self {
		use wgpu::util::DeviceExt;

		let camera = CameraUniform::new((window_size.0 as f32, window_size.1 as f32).into());

		let camera_buffer = bcknd.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: Some("Pipeline2d Camera Buffer"),
			contents: bytemuck::cast_slice(&[camera]),
			usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
		});

		let (uniform_bind_group_layout, uniform_bind_group) = Self::create_uniform_bind_group(&bcknd.device, &camera_buffer);

		let texture_queue = IndexSet::with_capacity(TEXTURE_QUEUE_SIZE as usize);

		// create fallback texture
		let (fallback_texture_view, fallback_sampler) = Self::create_fallback_tex(&bcknd.device, &bcknd.queue);

		let texture_bind_group_layout = bcknd.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
			label: Some("Pipeline2d BindGroupLayout"),
			entries: &[
				wgpu::BindGroupLayoutEntry {
					binding: 0,
					visibility: wgpu::ShaderStages::FRAGMENT,
					ty: wgpu::BindingType::Texture {
						sample_type: wgpu::TextureSampleType::Float { filterable: false },
						view_dimension: wgpu::TextureViewDimension::D2,
						multisampled: false,
					},
					count: Some(std::num::NonZeroU32::new(TEXTURE_QUEUE_SIZE).unwrap()),
				},
				wgpu::BindGroupLayoutEntry {
					binding: 1,
					visibility: wgpu::ShaderStages::FRAGMENT,
					ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
					count: Some(std::num::NonZeroU32::new(TEXTURE_QUEUE_SIZE).unwrap()),
				}
			]
		});

		let texture_bind_group = Self::create_texture_bind_group(
			bcknd,
			&texture_bind_group_layout,
			&texture_queue,
			&fallback_texture_view,
			&fallback_sampler
		).unwrap(); // the texture queue is empty, so i can't fail

		let pipeline_layout = bcknd.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
			label: None,
			bind_group_layouts: &[
				&uniform_bind_group_layout,
				&texture_bind_group_layout,
			],
			push_constant_ranges: &[],
		});

		let shader = bcknd.device.create_shader_module(wgpu::ShaderModuleDescriptor {
			label: Some("Pipeline2d Shader"),
			source: wgpu::ShaderSource::Wgsl(include_str!("quad_shader.wgsl").into()),
		});

		let pipeline = bcknd.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			label: Some("Pipeline2d Pipeline"),
			layout: Some(&pipeline_layout),
			vertex: wgpu::VertexState {
				module: &shader,
				entry_point: "vs_main",
				buffers: &[
					QuadVertex::describe(),
				]
			},
			fragment: Some(wgpu::FragmentState {
				module: &shader,
				entry_point: "fs_main",
				targets: &[
					Some(wgpu::ColorTargetState {
						format: surface_config.format,
						blend: Some(wgpu::BlendState::ALPHA_BLENDING),
						write_mask: wgpu::ColorWrites::ALL,
					})
				]
			}),
			primitive: wgpu::PrimitiveState {
				topology: wgpu::PrimitiveTopology::TriangleList,
				strip_index_format: None,
				front_face: wgpu::FrontFace::Ccw,
				cull_mode: Some(wgpu::Face::Back),
				unclipped_depth: false,
				polygon_mode: wgpu::PolygonMode::Fill,
				conservative: false,
			},
			depth_stencil: None,
			multisample: wgpu::MultisampleState {
				count: 1,
				mask: !0,
				alpha_to_coverage_enabled: false,
			},
			multiview: None,
		});

		let vertex_buffer = bcknd.device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("Pipeline2d Vertex Buffer"),
			size: VC as u64 * std::mem::size_of::<QuadVertex>() as u64,
			usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
			mapped_at_creation: false,
		});

		let index_buffer = bcknd.device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("Pipeline2d Vertex Buffer"),
			size: IC as u64 * std::mem::size_of::<u16>() as u64,
			usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::INDEX,
			mapped_at_creation: false,
		});


		Self {
			camera_buffer,
			uniform_bind_group,

			pipeline,
			vertex_buffer,
			index_buffer,
			vertices: Vec::with_capacity(VC),
			indices: Vec::with_capacity(IC),

			clear_queued: None,

			fallback_sampler,
			fallback_texture_view,
			texture_bind_group,
			texture_bind_group_layout,
			texture_queue,
		}
	}

	pub fn reconfigure(&self, bcknd: &WgpuBackend, window_size: (u32, u32)) {
		let camera = CameraUniform::new((window_size.0 as f32, window_size.1 as f32).into());
		self.update_camera(camera, &bcknd.queue)
	}

	pub fn update_camera(&self, camera: CameraUniform, queue: &wgpu::Queue) {
		queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[camera]));
	}

	fn queue_geometry(&mut self, vertices: &[QuadVertex], indices: &[u32], bcknd: &WgpuBackend, view: &wgpu::TextureView) -> Result<()> {
		if self.vertices.len() + vertices.len() > self.vertices.capacity() || self.indices.len() + indices.len() > self.indices.capacity() {
			self.flush(view, bcknd)?;
		}

		let offset = self.vertices.len() as u32;

		self.vertices.extend_from_slice(vertices);
		self.indices.extend(indices.iter().map(|s| s + offset));

		Ok(())
	}

	pub fn queue_texture(&mut self, bcknd: &WgpuBackend, tex: TextureId, end_view: &wgpu::TextureView) -> Result<u32> {
		if self.texture_queue.len() == TEXTURE_QUEUE_SIZE as usize {
			self.flush(end_view, bcknd)?;
		}

		let (index, _) = self.texture_queue.insert_full(tex);

		Ok(index as u32 + 1) // textures within the queue will be indexed starting from 1
	}

	pub fn queue_quad(&mut self, bcknd: &WgpuBackend, pos: (i32, i32), size: (u32, u32), color: Color, tex: Option<TextureId>, tex_coords: Option<TexCoords>, view: &wgpu::TextureView) -> Result<()> {
		use ultraviolet::*;

		let size = (size.0 as f32, size.1 as f32);

		let a = (pos.0 as f32, pos.1 as f32);
		let b = (a.0 + size.0, a.1 + size.1);

		let top_left =		Vec4::from([a.0, a.1, 0.0, 1.0]);
		let bottom_left =		Vec4::from([a.0, b.1, 0.0, 1.0]);
		let bottom_right =	Vec4::from([b.0, b.1, 0.0, 1.0]);
		let top_right =		Vec4::from([b.0, a.1, 0.0, 1.0]);

		let tex_id = if let Some(t) = tex {
			self.queue_texture(bcknd, t, view)?
		} else {
			0
		};

		let tex_coords = if let Some(t) = tex_coords {
			t
		} else {
			([0.0, 0.0], [0.0, 1.0], [1.0, 1.0], [1.0, 0.0])
		};

		let vertices = &[
			QuadVertex {
				pos: top_left.into(),
				color: color.into(),
				tex_coords: tex_coords.0,
				tex_id,
			},
			QuadVertex {
				pos: bottom_left.into(),
				color: color.into(),
				tex_coords: tex_coords.1,
				tex_id,
			},
			QuadVertex {
				pos: bottom_right.into(),
				color: color.into(),
				tex_coords: tex_coords.2,
				tex_id,
			},
			QuadVertex {
				pos: top_right.into(),
				color: color.into(),
				tex_coords: tex_coords.3,
				tex_id,
			},
		];

		let indices = &[
			0, 1, 2,
			0, 2, 3,
		];

		self.queue_geometry(vertices, indices, bcknd, view)
	}

	pub fn clear_queue(&mut self) {
		self.vertices.clear();
		self.indices.clear();
	}

	pub fn flush(&mut self, view: &wgpu::TextureView, bcknd: &WgpuBackend) -> Result<()> {
		let clear = self.clear_queued.take();

		if (self.vertices.is_empty() || self.indices.is_empty()) && clear.is_none() {
			return Ok(());
		}

		bcknd.queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&self.vertices));
		bcknd.queue.write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(&self.indices));

		if self.texture_queue.len() > 0 {

			self.texture_bind_group = Self::create_texture_bind_group(
				bcknd,
				&self.texture_bind_group_layout,
				&self.texture_queue,
				&self.fallback_texture_view,
				&self.fallback_sampler,
			)?;
		}

		let mut encoder = bcknd.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
			label: Some(std::any::type_name::<Self>()),
		});

		{
			let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
				label: Some("Render Pass"),
				color_attachments: &[
					Some(wgpu::RenderPassColorAttachment {
						view,
						resolve_target: None,

						ops: wgpu::Operations {
							load: match clear {
								Some(col) => wgpu::LoadOp::Clear(wgpu::Color { r: col.0, g: col.1, b: col.2, a: col.3 }),
								None => wgpu::LoadOp::Load,
							},
							store: true,
						},
					}),
				],
				depth_stencil_attachment: None,
			});

			render_pass.set_pipeline(&self.pipeline);
			render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
			render_pass.set_bind_group(1, &self.texture_bind_group, &[]);
			render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
			render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
			render_pass.draw_indexed(0..self.indices.len() as u32, 0, 0..1);
		}

		bcknd.queue.submit(std::iter::once(encoder.finish()));

		self.vertices.clear();
		self.indices.clear();

		Ok(())
	}

	pub fn set_clear(&mut self, color: Color) {
		self.clear_queued = Some(color);
	}
}


type TexCoords = ([f32; 2], [f32; 2], [f32; 2], [f32; 2]);


#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct QuadVertex {
	pub pos: [f32; 4],
	pub color: [f32; 4],
	pub tex_coords: [f32; 2],
	pub tex_id: u32,
}

impl QuadVertex {
	const ATTRIBS: [wgpu::VertexAttribute; 4] = wgpu::vertex_attr_array![
		0 => Float32x4,
		1 => Float32x4,
		2 => Float32x2,
		3 => Uint32
	];
	pub fn describe<'a>() -> wgpu::VertexBufferLayout<'a> {
		wgpu::VertexBufferLayout {
			array_stride: std::mem::size_of::<QuadVertex>() as wgpu::BufferAddress,
			step_mode: wgpu::VertexStepMode::Vertex,
			attributes: &Self::ATTRIBS,
		}
	}
}


#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
	view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
	fn new(screen_size: ultraviolet::Vec2) -> Self {
		let mat = ultraviolet::projection::orthographic_wgpu_dx(0.0, screen_size.x, screen_size.y, 0.0, 1.0, -1.0);

		Self {
			view_proj: mat.into(),
		}
	}
}

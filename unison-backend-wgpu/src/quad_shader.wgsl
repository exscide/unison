// Vertex shader

struct VertexInput {
	@location(0) position: vec4<f32>,
	@location(1) color: vec4<f32>,
	@location(2) tex_coords: vec2<f32>,
	@location(3) tex_id: u32,
};

struct VertexOutput {
	@builtin(position) clip_position: vec4<f32>,
	@location(0) color: vec4<f32>,
	@location(1) tex_coords: vec2<f32>,
	@location(2) tex_id: u32,
};

struct CameraUniform {
	view_proj: mat4x4<f32>,
};
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@vertex
fn vs_main(
	model: VertexInput,
) -> VertexOutput {
	var out: VertexOutput;
	out.clip_position = camera.view_proj * model.position;
	out.color = model.color;
	out.tex_coords = model.tex_coords;
	out.tex_id = model.tex_id;
	return out;
}


// Fragment shader

@group(1) @binding(0)
var tex: binding_array<texture_2d<f32>>;
@group(1) @binding(1)
var sam: binding_array<sampler>;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
	return in.color * textureSample(tex[in.tex_id], sam[in.tex_id], in.tex_coords);
}

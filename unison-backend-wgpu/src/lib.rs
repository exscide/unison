pub(crate) use unison_backend::types::*;

mod wgpu_backend;
pub use wgpu_backend::*;

mod quad_pipeline;
pub use quad_pipeline::*;

pub type Result<T> = std::result::Result<T, ()>;

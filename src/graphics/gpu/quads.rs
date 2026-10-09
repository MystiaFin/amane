mod batch;
mod clip;
mod draw;
mod pipeline;

use std::collections::HashMap;

use vello::wgpu::{BindGroup, BindGroupLayout, RenderPipeline, Sampler};

use super::atlas::Atlas;

pub use clip::{Clip, Clips};

// how many numbers one quad takes: Quad in quads.wgsl has seven rows of four
const QUAD_SIZE: usize = 9 * 4;

// what kind of quad it is, as quads.wgsl tells them apart
const SHAPE: f32 = 0.0;
const LETTER: f32 = 1.0;
const PICTURE: f32 = 2.0;

// quads next to each other that read the same image, drawn in one go
struct Run {
    // where the image lives, none for quads that read no image
    picture: Option<usize>,

    first: u32,
    count: u32,
}

/*
 * rounded rectangles, borders, letters and images gathered as they come and
 * drawn in one pass; this is most of what a shell shows, and the gpu draws
 * it far faster as plain triangles than vello does as vector shapes
 */
pub struct Quads {
    pipeline: RenderPipeline,
    erase_pipeline: RenderPipeline,
    inputs: BindGroupLayout,
    picture_inputs: BindGroupLayout,

    sampler: Sampler,

    atlas: Atlas,

    // images already on the gpu, keyed by where the loaded image lives
    pictures: HashMap<usize, BindGroup>,

    // what quads that read no image bind in the image's place
    blank: BindGroup,

    // every quad's numbers one after another, as quads.wgsl reads them
    waiting: Vec<f32>,
    runs: Vec<Run>,
}

// rounded rectangles, borders, letters and images, each drawn as one quad of two triangles

// what kind of quad it is, the same numbers as SHAPE, LETTER and PICTURE in quads.rs
const KIND_SHAPE: f32 = 0.0;
const KIND_LETTER: f32 = 1.0;
const KIND_PICTURE: f32 = 2.0;

/*
 * one quad, read from the numbers batch.rs writes, in the same order;
 * the fields come in groups of four numbers, so the order and sizes must not change
 */
struct Quad {
    // where it is on the canvas, in pixels
    position: vec2<f32>,
    size: vec2<f32>,

    // it only shows inside this rounded rectangle, and inside the inner clip too
    clip_position: vec2<f32>,
    clip_size: vec2<f32>,

    // not premultiplied, 0 to 1
    color: vec4<f32>,

    // 0 fills the shape
    border_thickness: f32,
    unused_1: f32,
    unused_2: f32,
    kind: f32,

    /*
     * where it reads from: a letter's texels in the atlas,
     * or the part of the image, from 0 to 1
     */
    source_start: vec2<f32>,
    source_end: vec2<f32>,

    inner_clip_position: vec2<f32>,
    inner_clip_size: vec2<f32>,

    // corner radii, top left, top right, bottom right, bottom left
    radius: vec4<f32>,
    clip_radius: vec4<f32>,
    inner_clip_radius: vec4<f32>,
}

@group(0) @binding(0) var<storage, read> quads: array<Quad>;

// width, height and two unused, in pixels
@group(0) @binding(1) var<uniform> canvas_size: vec4<f32>;

@group(0) @binding(2) var atlas: texture_2d<f32>;

@group(1) @binding(0) var picture: texture_2d<f32>;
@group(1) @binding(1) var picture_sampler: sampler;

// what the vertex step hands on to every pixel of the quad
struct Pixel {
    // where the gpu puts it, -1 to 1 across the canvas
    @builtin(position) gpu_position: vec4<f32>,

    // where it is on the canvas, in pixels
    @location(0) canvas_point: vec2<f32>,

    // where it reads from in the atlas or the image
    @location(1) source_point: vec2<f32>,

    @location(2) @interpolate(flat) quad_index: u32,
}

@vertex
fn vertex(@builtin(vertex_index) vertex: u32, @builtin(instance_index) quad_index: u32) -> Pixel {
    let quad = quads[quad_index];

    // the two triangles' corners, from 0 to 1 across the quad
    var corners = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(1.0, 1.0),
    );

    let corner = corners[vertex];

    // a shape's soft edge reaches half a pixel past it, letters and images fill theirs exactly
    var margin = 1.0;

    if quad.kind != KIND_SHAPE {
        margin = 0.0;
    }

    let grown_position = quad.position - margin;
    let grown_size = quad.size + margin * 2.0;

    let canvas_point = grown_position + corner * grown_size;

    // the gpu wants -1 to 1 from left to right, and 1 to -1 from top to bottom
    let gpu_x = canvas_point.x / canvas_size.x * 2.0 - 1.0;
    let gpu_y = 1.0 - canvas_point.y / canvas_size.y * 2.0;

    var pixel: Pixel;

    pixel.gpu_position = vec4<f32>(gpu_x, gpu_y, 0.0, 1.0);
    pixel.canvas_point = canvas_point;
    pixel.source_point = mix(quad.source_start, quad.source_end, corner);
    pixel.quad_index = quad_index;

    return pixel;
}

// how far point is from the rounded rectangle's edge, below zero inside
fn distance_to_rounded_rectangle(
    point: vec2<f32>,
    position: vec2<f32>,
    size: vec2<f32>,
    radius: vec4<f32>,
) -> f32 {
    let half_size = size * 0.5;
    let middle = position + half_size;

    let from_middle = point - middle;

    // the corner of the quarter the point is in, y grows downward
    let on_right = from_middle.x > 0.0;
    let on_bottom = from_middle.y > 0.0;

    let top_radius = select(radius.x, radius.y, on_right);
    let bottom_radius = select(radius.w, radius.z, on_right);
    let corner_radius = select(top_radius, bottom_radius, on_bottom);

    // a corner can't be rounder than half the shorter side
    let safe_radius = clamp(corner_radius, 0.0, min(half_size.x, half_size.y));

    // how far past the edge on each side, with the edge pulled in by the corner
    let past_edge = abs(from_middle) - half_size + safe_radius;

    let inside = min(max(past_edge.x, past_edge.y), 0.0);
    let outside = length(max(past_edge, vec2<f32>(0.0)));

    return inside + outside - safe_radius;
}

// how much of the pixel the shape covers, 0 to 1
fn coverage(distance: f32) -> f32 {
    return clamp(0.5 - distance, 0.0, 1.0);
}

fn shadow_coverage(distance: f32, blur: f32) -> f32 {
    return clamp(0.5 - distance / blur, 0.0, 1.0);
}

@fragment
fn fragment(pixel: Pixel) -> @location(0) vec4<f32> {
    let quad = quads[pixel.quad_index];

    // sampled for every quad, since sampling has to happen outside of branches
    let picture_color = textureSample(picture, picture_sampler, pixel.source_point);

    // how much each clip lets the pixel show, 0 to 1
    let outer_clip_distance = distance_to_rounded_rectangle(
        pixel.canvas_point,
        quad.clip_position,
        quad.clip_size,
        quad.clip_radius,
    );

    let inner_clip_distance = distance_to_rounded_rectangle(
        pixel.canvas_point,
        quad.inner_clip_position,
        quad.inner_clip_size,
        quad.inner_clip_radius,
    );

    let clip_shows = coverage(outer_clip_distance) * coverage(inner_clip_distance);

    // images hold premultiplied colors already, the color's alpha fades them
    if quad.kind == KIND_PICTURE {
        return picture_color * quad.color.a * clip_shows;
    }

    // how much of the pixel the letter or shape covers, 0 to 1
    var covered = 0.0;

    if quad.kind == KIND_LETTER {
        let atlas_texel = vec2<i32>(floor(pixel.source_point));

        covered = textureLoad(atlas, atlas_texel, 0).r;
    } else {
        let distance = distance_to_rounded_rectangle(
            pixel.canvas_point,
            quad.position,
            quad.size,
            quad.radius,
        );

        covered = coverage(distance);

        let thickness = quad.border_thickness;

        // a border is the shape minus the same shape pulled in by the thickness
        if thickness > 0.0 {
            let hole_position = quad.position + thickness;
            let hole_size = quad.size - thickness * 2.0;
            let hole_radius = max(quad.radius - thickness, vec4<f32>(0.0));

            var hole = 0.0;

            if hole_size.x > 0.0 && hole_size.y > 0.0 {
                let hole_distance = distance_to_rounded_rectangle(
                    pixel.canvas_point,
                    hole_position,
                    hole_size,
                    hole_radius,
                );

                hole = coverage(hole_distance);
            }

            covered = covered * (1.0 - hole);
        }
    }

    covered = covered * clip_shows;

    let alpha = quad.color.a * covered;

    return vec4<f32>(quad.color.rgb * alpha, alpha);
}

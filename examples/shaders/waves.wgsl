// amane gives uv (0 to 1 across the rectangle), size in pixels and time in seconds
@fragment
fn main(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
    let wave = sin(uv.x * 12.0 + time * 2.0) * 0.5 + 0.5;

    return vec4<f32>(uv.x, wave, 1.0 - uv.y, 1.0);
}

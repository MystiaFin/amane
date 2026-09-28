// amane's half of every custom shader: it places the rectangle and hands uv to the user's half

struct Placement {
    // x, y, width and height of the rectangle before the transform
    rect: vec4<f32>,

    // the transform's two rows: sx, kx, tx and ky, sy, ty
    row_x: vec4<f32>,
    row_y: vec4<f32>,

    // the canvas width and height in real pixels
    canvas: vec4<f32>,
}

@group(0) @binding(2) var<uniform> placement: Placement;

struct Corner {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vertex(@builtin(vertex_index) index: u32) -> Corner {
    // two triangles that make up the rectangle
    var corners = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(1.0, 1.0),
    );

    let uv = corners[index];

    let local = placement.rect.xy + uv * placement.rect.zw;

    let x = dot(placement.row_x.xyz, vec3<f32>(local, 1.0));
    let y = dot(placement.row_y.xyz, vec3<f32>(local, 1.0));

    // pixels grow right and down, the gpu's own coordinates grow right and up from -1 to 1
    let gpu_x = x / placement.canvas.x * 2.0 - 1.0;
    let gpu_y = 1.0 - y / placement.canvas.y * 2.0;

    return Corner(vec4<f32>(gpu_x, gpu_y, 0.0, 1.0), uv);
}

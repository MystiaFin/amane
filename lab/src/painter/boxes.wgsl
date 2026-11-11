// the screen goes from -1 to 1 both ways, with 0 in the middle
const CORNERS = array(
    vec2(-0.5, -0.5),
    vec2(0.5, -0.5),
    vec2(-0.5, 0.5),
    vec2(-0.5, 0.5),
    vec2(0.5, -0.5),
    vec2(0.5, 0.5),
);

// runs once per corner, and says where that corner goes
@vertex
fn place(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
    let corner = CORNERS[index];

    return vec4(corner, 0.0, 1.0);
}

// runs once per pixel inside the box, and says its color
@fragment
fn color() -> @location(0) vec4<f32> {
    return vec4(1.0, 1.0, 1.0, 1.0);
}

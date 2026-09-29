// draws a soft spot at each position the rectangle hands over in values
// values[0].x is how many spots, then one row per spot: x, y, radius
@fragment
fn main(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
    let point = uv * size;

    let count = u32(values[0].x);

    var light = 0.0;

    for (var index = 1u; index <= count; index++) {
        let spot = values[index];

        let distance = length(point - spot.xy);

        light = max(light, 1.0 - smoothstep(spot.z - 1.0, spot.z + 1.0, distance));
    }

    return vec4<f32>(0.54, 0.71, 0.98, light);
}

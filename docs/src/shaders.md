# Shaders

A shader is a small program that runs on the GPU and works out the color of every pixel in a rectangle. Use one for effects no built-in fill can do: animated backgrounds, glows, noise, and custom gradients.

## An animated shader

`~/.config/amane/waves.wgsl`:

```wgsl
@fragment
fn main(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
    let wave = sin(uv.x * 12.0 + time * 2.0) * 0.5 + 0.5;

    return vec4<f32>(uv.x, wave, 1.0 - uv.y, 1.0);
}
```

`main.rs`:

```rust
use amane::{App, LayerWindow, Rectangle};

fn main() {
    App::new().window(view).run();
}

fn view() -> LayerWindow {
    LayerWindow::new().width(200.0).height(200.0).child(
        Rectangle::new()
            .width(200.0)
            .height(200.0)
            .radius(24.0)
            .shader("/home/you/.config/amane/waves.wgsl"),
    )
}
```

The rectangle shows moving colored waves, cut to its rounded corners.

## Shader inputs

The GPU runs `main` once for every pixel of the rectangle, all at the same time. Each run gets that pixel's position and returns its color. Amane gives every shader these inputs:

| Input | Type | Meaning |
|---|---|---|
| `uv` | `vec2<f32>` | where the pixel is, from `(0, 0)` at the top left to `(1, 1)` at the bottom right |
| `size` | `vec2<f32>` | the rectangle's size in pixels |
| `time` | `f32` | seconds since the first shader was drawn |
| `values` | `array<vec4<f32>, 16>` | numbers you pass from Rust, see [Passing values from Rust](#passing-values-from-rust) |

The color you return is red, green, blue, and alpha, each from 0 to 1.

The shader is drawn over the rectangle's fill, inside its shape. The rectangle's radius, border, opacity, and transforms all still apply.

## Animated shaders and frames

Amane checks whether your shader's source mentions the word `time`. If it does, the window draws a new frame continuously while the shader is on screen, so the animation runs. If it doesn't, the shader is drawn once, and only redrawn when something else changes.

Any `time` counts, even in a comment. Leave the word out of shaders that don't animate, so they don't keep your GPU busy.

## Passing values from Rust

`shader_values` passes up to 16 rows of 4 numbers from your view to the shader, as `values`. Rows you don't pass are 0.

```rust,ignore
Rectangle::new()
    .width(420.0)
    .height(200.0)
    .shader("/home/you/.config/amane/spots.wgsl")
    .shader_values(vec![[2.0, 0.0, 0.0, 0.0], [60.0, 100.0, 30.0, 0.0], [200.0, 80.0, 50.0, 0.0]])
```

```wgsl
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
```

The values come from the view, so they can come from a Service or an Animation, like an audio level or a hover position. More than 16 rows panics.

## GLSL

Files ending in `.glsl` or `.frag` are read as GLSL. Amane adds the `#version` line and the inputs for you. Write the result to `color`:

```glsl
void main() {
    vec2 pixel = uv * size;
    vec2 center = size / 2.0;

    float rings = sin(distance(pixel, center) * 0.15 - time * 3.0) * 0.5 + 0.5;

    color = vec4(rings, 0.3, 1.0 - rings, 1.0);
}
```

`values` is available in GLSL too, as `vec4 values[16]`.

Any other extension is read as WGSL.

## Limitations

- **The entry point must be called `main`.**
- **A missing shader file stops the shell.** A shader is loaded when it's first drawn, and if the file can't be read, the shell exits with an error. A shader that doesn't compile doesn't stop the shell. It prints `amane: gpu error: ...` to the terminal instead, and the effect won't draw correctly. Test new shaders with `amane dev`, where you can see that output.
- **A shader file is read once.** Editing it while the shell runs doesn't change it. `amane dev` doesn't watch shader files either, unless they're inside `src/`, so restart the shell to see changes.
- Use absolute paths, for the same reason as [images](images.md).

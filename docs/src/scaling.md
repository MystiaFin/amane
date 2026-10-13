# Scaling

Set a scale factor on `App` to make the whole shell larger or smaller:

```rust,ignore
App::new().scale_factor(1.4).window(bar).run();
```

The default is `1.0`. The factor must be positive and finite; invalid values panic when you call `scale_factor`.

All layer windows, normal windows, and lock screens use the same factor, including windows opened later. Widget sizes, text, padding, borders, effects, window margins, reserved space, and input regions scale together. A bar with height `30.0` requests 42 surface pixels at factor `1.4`. `Full` still fills the monitor.

The compositor's monitor scale also applies. On a monitor scaled to `1.25`, an app factor of `1.4` draws at `1.75` device pixels per widget unit. Amane keeps its buffer at the monitor's resolution, so it does not scale the finished image a second time.

`window_size()` and `Monitor` dimensions use widget units, before the app factor. Pointer motion and drag callbacks do too, including inside transformed widgets. You can keep the same view code at any scale. Monitor dimensions are rounded to whole units; use `Full` when a window should stretch exactly between monitor edges.

## Choose the scale when starting

The factor is set when the app starts. To change it from a script without editing the shell, read an environment variable in your `main`:

```rust,ignore
let factor = std::env::var("AMANE_SCALE_FACTOR")
    .map(|value| value.parse::<f32>().expect("invalid AMANE_SCALE_FACTOR"))
    .unwrap_or(1.0);

App::new().scale_factor(factor).window(bar).run();
```

Then start that shell with `AMANE_SCALE_FACTOR=1.4`. Restart it to use another factor. The variable is read by this example code, so add it to your shell if you want this behavior.

Try the example on a compositor:

```sh
AMANE_SCALE_FACTOR=1.4 cargo run --example scale
```

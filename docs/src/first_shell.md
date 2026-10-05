# Your First Shell

This page gets a bar on your screen, then changes it while it runs.

## 1. Create the config

```sh
amane startup
```

This creates `~/.config/amane/src/main.rs` (or `$XDG_CONFIG_HOME/amane/src/main.rs`) with a small bar in it. If the file already exists, `startup` stops and leaves it alone.

## 2. Start it in dev mode

```sh
amane dev
```

The first build takes a few minutes, because it compiles Amane and all its dependencies. After that, the bar appears at the top of your screen.

Leave `amane dev` running. Every time you save `main.rs`, it rebuilds and restarts the bar. If a build fails, the old bar stays on screen while you fix the error.

Only one Amane shell can run at a time. If your shell is already running from `amane run`, stop it before starting `amane dev`.

## 3. Read the file

Open `~/.config/amane/src/main.rs`:

```rust
use amane::{App, Color, Full, Layer, LayerWindow, Parent, Rectangle, Text, Vertical};

fn main() {
    App::new().window(view).run();
}

fn view() -> LayerWindow {
    LayerWindow::new()
        .width(Full)
        .height(30.0)
        .anchor_vertical(Vertical::Top)
        .layer(Layer::Top)
        .child(
            Rectangle::new()
                .width(Parent)
                .height(Parent)
                .fill(Color::BLUE)
                .child(Text::new("hello from amane").size(20.0).color(Color::WHITE)),
        )
}
```

Top to bottom:

- `use amane::{...}` brings in every name the file uses.
- `main` builds an `App`, gives it one window, and runs it. `run()` never returns while the shell is open.
- `.window(view)` passes the function `view` itself, without calling it. Amane calls it whenever the window needs to be drawn.
- `view` describes the window:
  - `.width(Full)` makes it as wide as the monitor.
  - `.height(30.0)` makes it 30 pixels tall.
  - `.anchor_vertical(Vertical::Top)` sticks it to the top edge.
  - `.layer(Layer::Top)` puts it above normal windows.
- Inside the window is a `Rectangle` that fills it (`Parent` means "as big as my parent") and is painted blue.
- Inside the rectangle is a `Text`.

## 4. Change it

With `amane dev` still running, change the color and the text:

```rust
use amane::{App, Color, Full, Layer, LayerWindow, Parent, Rectangle, Text, Vertical};

fn main() {
    App::new().window(view).run();
}

fn view() -> LayerWindow {
    LayerWindow::new()
        .width(Full)
        .height(30.0)
        .anchor_vertical(Vertical::Top)
        .layer(Layer::Top)
        .child(
            Rectangle::new()
                .width(Parent)
                .height(Parent)
                .fill("#1e1e2e")
                .child(Text::new("my shell").size(16.0).color(Color::from("#cdd6f4"))),
        )
}
```

Save, and the bar restarts with the new look.

`.fill` takes a hex string directly. `.color` on `Text` takes a `Color`, so a hex string goes through `Color::from`.

## 5. Run it for real

`amane dev` stops your shell when you stop it. When you're happy with your shell, compile it once:

```sh
amane compile
```

Then start it with:

```sh
amane run
```

`amane run` never builds. It only starts the shell `amane compile` saved, so after editing your shell, run `amane compile` again. To start your shell when you log in, run `amane run` from your compositor's startup config. For niri:

```kdl
spawn-at-startup "amane" "run"
```

## Next

[Views](views.md) explains what `view` really is and why it's a function. Read it before writing anything bigger.

# Palette

`Palette` takes the main colors out of an image, usually your wallpaper, so your shell's colors can follow it.

## A themed bar

```rust
use amane::{App, Full, LayerWindow, Palette, Parent, Rectangle, Service, Text};

fn main() {
    Palette::write().open("/home/you/Pictures/wall.jpg", 16);

    App::new().window(view).run();
}

fn view() -> LayerWindow {
    let palette = Palette::read();

    LayerWindow::new().width(Full).height(30.0).child(
        Rectangle::new()
            .width(Parent)
            .height(Parent)
            .fill(palette.background())
            .child(Text::new("themed").color(palette.foreground())),
    )
}
```

## How colors are picked and updated

- `open(path, count)` reads the image and picks `count` colors from it. It reads the image right away, so the very first frame already has the colors. 16 is enough for a whole shell's theme.
- `open` changes the Palette, so it's called on `write()`, and every window that reads the Palette redraws. That's the one place it's fine to call `write()` outside an input handler: in `main`, before the shell starts, or in an IPC handler.
- The Palette checks the file twice a second. When the image changes, or you `open` a new path, the colors are picked again on a background thread, and the windows redraw.

So a wallpaper script that overwrites the same file re-themes your shell on its own.

## Reference

| Function | Gives or does |
|---|---|
| `open(path, count)` | reads an image and picks `count` colors. Call it on `Palette::write()`. |
| `colors()` | every color picked, most common first |
| `dominant()` | the color the image shows most |
| `accent()` | the most vivid color, for highlights and the focused workspace |
| `background()` | the darkest color, so light text always reads on it |
| `foreground()` | a color readable on `background()`, tinted by the image when possible |
| `on_accent()` | white or black, whichever reads better on `accent()` |
| `light()` | `true` when the image is light overall |

Before an image is opened, the Palette holds a set of default colors.

## Changing the wallpaper from a keybind

```rust,ignore
fn main() {
    App::new()
        .ipc("wallpaper", |arguments| {
            let Some(path) = arguments.first() else {
                return String::from("usage: wallpaper <path>");
            };

            Palette::write().open(path, 16);

            format!("palette from {path}")
        })
        .window(bar)
        .run();
}
```

```sh
amane ipc call wallpaper ~/Pictures/new.jpg
```

Here `~` works, because your shell (fish, bash, zsh) expands it before `amane` sees it.

`open` reads the image on the spot, and IPC handlers run on the thread that draws, so the shell pauses for a moment on a very large image. If that bothers you, have your wallpaper script copy the new image over one fixed path instead, and `open` that path once in `main`. The Palette notices the change on its own and reads it on a background thread.

# Images

Images are drawn as a rectangle's fill. This page covers fitting them, loading them, and keeping big ones cheap.

## An image fill

```rust
use amane::{App, Image, LayerWindow, Rectangle};

fn main() {
    App::new().window(view).run();
}

fn view() -> LayerWindow {
    LayerWindow::new().width(300.0).height(200.0).child(
        Rectangle::new()
            .width(300.0)
            .height(200.0)
            .radius(16.0)
            .fill(Image::cover("/usr/share/backgrounds/default.png")),
    )
}
```

The image fills the rectangle, follows its rounded corners, and works with `border`, `shadow`, and `opacity` like any other fill.

## Fitting an image

| Fit | Effect |
|---|---|
| `Image::cover(path)` | fills the whole rectangle, cutting off what spills past the edges |
| `Image::contain(path)` | shows the whole image, leaving the rest of the rectangle empty |
| `Image::stretch(path)` | fills the rectangle exactly, squashing the image to match |

`cover` is right for wallpapers and album art. `contain` is right for icons.

## Formats and paths

- PNG and JPEG are recognized by their contents, whatever the file name says. SVG is recognized by the `.svg` extension.
- Use absolute paths. A relative path is relative to the folder the shell was started from, which depends on how it was started. `~` isn't expanded.

## How images load

Decoding a big image takes long enough to drop frames, so Amane decodes on a separate thread:

1. The first time a view asks for a file, the rectangle is drawn without the image, and decoding starts.
2. When decoding finishes, the windows waiting for that image redraw, now with the image.
3. After that, the decoded image is kept in memory, keyed by its path, and appears instantly.

A file that can't be read or decoded never appears, and doesn't crash the shell.

Because images are kept by path, changing a file on disk doesn't change what's shown. To show a new wallpaper, use a new path.

To show something else while an image loads, ask whether it's ready:

```rust,ignore
// players give a file:// link, and Image wants a plain path
let art = media.art_url().strip_prefix("file://").unwrap_or("");

let cover = if Image::loaded(art) {
    Rectangle::new().width(64.0).height(64.0).fill(Image::cover(art))
} else {
    Rectangle::new().width(64.0).height(64.0).fill("#313244")
};
```

`Image::loaded` also starts the decode if nothing asked for it yet.

## Thumbnails

A 4K wallpaper shown as a 200-pixel preview still keeps all 8 million pixels in memory. `thumbnail` keeps a small copy instead:

```rust,ignore
Image::cover(path).thumbnail(200, 120)
```

The copy is just big enough to cover 200 by 120 pixels. The full-size image is dropped right after shrinking. Use it for wallpaper pickers, icon grids, and anything else that shows many images at once.

## Blurred backgrounds

```rust,ignore
Image::cover(wallpaper).thumbnail(64, 36).blurred(4)
```

`blurred` blurs the decoded copy once, when it's loaded, so it costs nothing per frame. Blurring a tiny thumbnail and stretching it to full screen is a cheap way to get a smooth, frosted background, for example for a lock screen.

The radius is counted in the copy's own pixels, so with a small thumbnail, small numbers already blur a lot.

## App icons

The [Apps](apps.md) Service finds each program's icon file for you:

```rust,ignore
let icon = Rectangle::new().width(24.0).height(24.0);

let icon = match app.icon_path() {
    Some(path) => icon.fill(Image::contain(path)),
    None => icon,
};
```

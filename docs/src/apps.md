# Apps

`Apps` lists the programs installed on your system, from their `.desktop` files, with their names and icons. Use it to build an app launcher.

## A launcher

```rust
use amane::{App, Apps, Color, Column, Image, LayerWindow, Pointer, Rectangle, Row, ScrollArea, Service, Text, Widget, children};

fn main() {
    App::new().window(view).run();
}

fn view() -> LayerWindow {
    let apps = Apps::read();

    let mut rows: Vec<Box<dyn Widget>> = Vec::new();

    for app in apps.list() {
        let launched = app.clone();

        let icon = Rectangle::new().width(24.0).height(24.0);

        let icon = match app.icon_path() {
            Some(path) => icon.fill(Image::contain(path)),
            None => icon,
        };

        let row = Rectangle::new()
            .width(300.0)
            .height(32.0)
            .fill("#1e1e2e")
            .cursor(Pointer)
            .on_click(move |_| launched.launch())
            .child(Row::new(children![
                icon,
                Text::new(app.name()).size(16.0).color(Color::WHITE),
            ]));

        rows.push(Box::new(row));
    }

    LayerWindow::new()
        .width(300.0)
        .height(500.0)
        .child(ScrollArea::new("apps", Column::new(rows)))
}
```

`App` (the shell) and `Apps` (the Service) are different things. `app` in the loop is one `DesktopApp`.

## Reference

`list()` gives every program, sorted by name, without the ones marked hidden or not to be shown in menus. Each `DesktopApp` has:

| Function | Gives or does |
|---|---|
| `name()` | its name, like "Firefox" |
| `description()` | its comment, or a generic name like "Web Browser", or `None` |
| `exec()` | its command line, without the `%f`-style placeholders |
| `icon()` | its icon's name in the icon theme, like `"firefox"`, or `None` |
| `icon_path()` | the icon's image file, a PNG when the theme has one, or `None` |
| `launch()` | starts the program, without waiting for it |

## First load and rescans

Finding icons means walking every icon theme on your system, which takes a few seconds. So `list()` is empty at first, and fills in once the first scan is done. The window redraws by itself when it does.

After that, the list is scanned again every 30 seconds, so programs you install while the shell runs show up.

## Searching

Filter the list with a `TextInput` ([Text Input](text_input.md)):

```rust,ignore
let query = search.query.to_lowercase();

for app in apps.list() {
    if !app.name().to_lowercase().contains(&query) {
        continue;
    }

    // add a row
}
```

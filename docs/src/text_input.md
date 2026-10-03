# Text Input

`TextInput` is a one-line box you can type in: a search field, a password field, a command prompt.

## A text field

```rust
use amane::{App, Keyboard, LayerWindow, Rectangle, TextInput};

fn main() {
    App::new().window(view).run();
}

fn view() -> LayerWindow {
    LayerWindow::new()
        .width(300.0)
        .height(40.0)
        .keyboard(Keyboard::OnDemand)
        .child(
            Rectangle::new()
                .width(300.0)
                .height(40.0)
                .fill("#ffffff")
                .padding(8.0)
                .child(
                    TextInput::new("name")
                        .placeholder("your name")
                        .on_submit(|text| println!("hello, {text}")),
                ),
        )
}
```

Click the box, type, and press Enter.

## Focus and stored text

- The window asks for keyboard focus with `.keyboard(Keyboard::OnDemand)`. Without it, the compositor never sends the window any keys, and the input can't be typed in ([Layer Windows](layer_windows.md)).
- Clicking the input gives it focus. From then on, typed keys go to it.
- The view is rebuilt on every frame, so the input can't keep its own text. Amane keeps it for you, under the name you passed to `TextInput::new`, here `"name"`. Each input needs its own name.

## Options

```rust,ignore
TextInput::new("search")
    .size(18.0)                       // text size, 16 by default
    .color(Color::WHITE)              // text color, black by default
    .width(240.0)                     // Parent by default
    .placeholder("search apps")       // shown while it's empty
    .password()                       // shows dots instead of the text
    .focused()                        // takes the keys as soon as it's drawn, no click needed
    .on_change(|text| ...)            // runs after every change, with the whole new text
    .on_submit(|text| ...)            // runs when Enter is pressed
```

`focused` is what you want for a launcher: open it, start typing. Pair it with `Keyboard::Exclusive` on the window.

Text that's longer than the input is cut off at its edge.

## Reading what was typed

The input owns its text. To use the text anywhere else, copy it into a Service in `on_change`:

```rust
use std::time::Duration;

use amane::{App, Column, Keyboard, LayerWindow, Service, Text, TextInput, children};

struct Search {
    query: String,
}

impl Service for Search {
    fn new() -> Self {
        Self { query: String::new() }
    }

    fn interval() -> Duration {
        Duration::from_secs(3600)
    }
}

fn main() {
    App::new().window(view).run();
}

fn view() -> LayerWindow {
    let search = Search::read();

    LayerWindow::new()
        .width(300.0)
        .height(80.0)
        .keyboard(Keyboard::Exclusive)
        .child(Column::new(children![
            TextInput::new("search")
                .focused()
                .on_change(|text| Search::write().query = text),
            Text::new(format!("searching for: {}", search.query)),
        ]))
}
```

## Setting the text

`TextInput::set_text` replaces what an input holds, by name, and puts the cursor at the end:

```rust,ignore
TextInput::set_text("search", "");          // clear it, for example after Enter
TextInput::set_text("search", "firefox");   // fill it in
```

Enter doesn't clear the input on its own. Call `set_text` with an empty string in `on_submit` if you want that.

## Keys

While an input has focus:

- Letters, `Space`, `Backspace`, `Left`, `Right`, `Home`, and `End` edit the text.
- `Enter` calls `on_submit`.
- `Escape` takes focus away, and then also reaches the window's `on_key`.
- `Up`, `Down`, and `Tab` skip the input and go straight to the window's `on_key`, so a list under a search box can be moved through with the arrow keys.

Holding a key down doesn't repeat it yet.

## Password fields

```rust,ignore
TextInput::new("password")
    .placeholder("password")
    .password()
    .on_submit(|password| Lock::unlock(&password))
```

`password` only changes what's drawn. The text passed to `on_change` and `on_submit` is the real text. See [Lock Screen](lock_screen.md) for a full example.

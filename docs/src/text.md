# Text

`Text` draws one or more lines of text. This page covers size, color, fonts, weight, and what happens when text doesn't fit.

## Drawing text

```rust
use amane::{App, Color, LayerWindow, Text};

fn main() {
    App::new().window(view).run();
}

fn view() -> LayerWindow {
    LayerWindow::new()
        .width(300.0)
        .height(40.0)
        .child(Text::new("hello").size(20.0).color(Color::WHITE))
}
```

`Text::new` takes a `&str`, a `String`, or anything else that turns into a `String`, so `format!` works directly:

```rust,ignore
Text::new(format!("battery {}%", battery.percent()))
```

The defaults are size 16, black, regular weight, and your system's `sans-serif` font.

## Color

```rust,ignore
Text::new("hello").color(Color::WHITE)
Text::new("hello").color(Color::from("#cdd6f4"))
```

`color` takes a `Color`. Unlike `Rectangle::fill`, it doesn't take a hex string directly, so wrap hex strings in `Color::from`.

## Fonts

Fonts are found through fontconfig, so any font installed on your system works, by its family name:

```rust,ignore
App::new().font("Inter").window(view).run()   // the default for all text
Text::new("12:00").font("JetBrains Mono")      // just this text
```

If a letter isn't in the font, like Japanese text in a Latin font, Amane looks for it in Noto Sans, then in Noto Sans CJK JP. A letter no installed font has is skipped.

## Weight

```rust,ignore
Text::new("bold").weight(Weight::Bold)
Text::new("light").weight(300)
```

`Weight` has `Thin`, `ExtraLight`, `Light`, `Regular`, `Medium`, `SemiBold`, `Bold`, `ExtraBold`, and `Black`. A number from 100 to 900 picks the closest one, like in CSS. The font has to have that weight installed. Most font families come in several separate files, one per weight.

## Long text: eliding and wrapping

By default, text is one line, and it's exactly as wide as its letters. In a `Row`, it takes the space it needs, and `Parent`-sized neighbors get the rest.

When the text is longer than the room it's given, it runs past the edge. Two options change that:

```rust,ignore
Text::new(long).elide()                     // one line, cut off with "…"
Text::new(long).wrap()                      // as many lines as it needs
Text::new(long).wrap().max_lines(2).elide() // up to two lines, then "…"
```

With `wrap` or `elide`, the text's width becomes `Parent`: it takes all the width it's given, and fits itself into it. So put it somewhere with a set width, like a fixed-width rectangle or a `Parent`-sized section of a row.

Height works like this:

| Rules | Height |
|---|---|
| neither | one line |
| `wrap` with `max_lines(n)` | `n` lines |
| `wrap` alone | `Parent`, because the number of lines depends on the width, which isn't known until layout is done |

When you need the height of wrapped text yourself, for example to size a notification card around it, ask for it with a width:

```rust,ignore
let body = Text::new(notification.body()).wrap();

let height = body.height_in(360.0);
```

## Icon fonts

Text is normally measured by the space its letters take when typed, including some room on each side. For icon fonts, that extra room makes icons look a little off-center. `tight` measures the letters' actual shapes instead:

```rust,ignore
Text::new("\u{f240}").font("Symbols Nerd Font").size(18.0).tight()
```

## Example

```rust
use amane::{App, Column, LayerWindow, Text, Weight, children};

fn main() {
    App::new().window(view).run();
}

fn view() -> LayerWindow {
    let long = "the quick brown fox jumps over the lazy dog, then keeps running far past the edge";

    LayerWindow::new()
        .width(260.0)
        .height(400.0)
        .child(Column::new(children![
            Text::new("regular"),
            Text::new("bold").weight(Weight::Bold),
            Text::new("light, picked by number").weight(300),
            Text::new(long).elide(),
            Text::new(long).wrap().max_lines(2).elide(),
            Text::new(long).wrap(),
        ]))
}
```

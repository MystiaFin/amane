# Media Players

`Media` shows what's playing and controls playback, in any player that supports MPRIS: Spotify, mpv, Firefox and Chromium tabs, VLC, and most others.

It polls once a second, because players don't announce their position while a track plays.

## A now-playing widget

```rust
use amane::{App, Button, Full, LayerWindow, Media, Parent, Rectangle, Row, Service, Text, children};

fn main() {
    App::new().window(view).run();
}

fn view() -> LayerWindow {
    let media = Media::read();

    let symbol = if media.playing() { "pause" } else { "play" };

    let seconds = media.position().as_secs();
    let total = media.length().as_secs();

    let label = format!(
        "{} - {}  {}:{:02} / {}:{:02}",
        media.artist(),
        media.title(),
        seconds / 60,
        seconds % 60,
        total / 60,
        total % 60,
    );

    LayerWindow::new().width(Full).height(30.0).child(Row::new(children![
        button("prev", |_| Media::previous()),
        button(symbol, |_| Media::play_pause()),
        button("next", |_| Media::next()),
        Rectangle::new().width(Parent).height(Parent).child(Text::new(label)),
    ]))
}

fn button(label: &str, clicked: fn(Button)) -> Rectangle {
    Rectangle::new()
        .width(60.0)
        .height(Parent)
        .on_click(clicked)
        .child(Text::new(label))
}
```

## The active player

With several players open, the functions on `Media` itself show and control one of them, the **active** player:

1. the one that's playing,
2. or else the one shown last,
3. or else the first one.

| Function | Gives or does |
|---|---|
| `title()` | the track's title, empty when no player is open |
| `artist()` | the track's artist |
| `art_url()` | a link to the cover art, usually `file://...` or `https://...` |
| `playing()` | `true` while playing |
| `position()` | how far into the track, as a `Duration` |
| `length()` | the track's length, as a `Duration` |
| `Media::play_pause()` | toggles playback |
| `Media::next()` | skips to the next track |
| `Media::previous()` | goes back to the previous track |

`Duration` is `std::time::Duration`. Use `.as_secs()` to get whole seconds.

To show the cover art, strip the `file://` from `art_url()` and use it as an image ([Images](images.md)). `https://` links can't be shown.

## Every player

`players()` lists every open player, in the same order on every poll, and `active()` gives the active one, or `None` when no player is open. Each `MediaPlayer` has the same functions as above, plus `identity()` (its display name, like "Spotify"), `name()` (its D-Bus name), and its own `play_pause()`, `next()`, and `previous()`:

```rust
use amane::{App, Column, Full, LayerWindow, Media, MediaPlayer, Parent, Rectangle, Service, Text, Widget};

fn main() {
    App::new().window(view).run();
}

fn view() -> LayerWindow {
    let media = Media::read();

    let mut rows: Vec<Box<dyn Widget>> = Vec::new();

    for player in media.players() {
        rows.push(Box::new(player_row(player)));
    }

    LayerWindow::new().width(Full).height(90.0).child(Column::new(rows))
}

fn player_row(player: &MediaPlayer) -> Rectangle {
    let label = format!("{}: {} - {}", player.identity(), player.artist(), player.title());

    // the closure keeps its own copy, so it keeps controlling this player
    let target = player.clone();

    Rectangle::new()
        .width(Parent)
        .height(30.0)
        .on_click(move |_| target.play_pause())
        .child(Text::new(label))
}
```

The control functions run on a background thread and return right away.

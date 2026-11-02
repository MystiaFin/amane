# Audio

`Audio` reads and controls the volume of your default speaker and microphone. It talks to PulseAudio, which also covers PipeWire through `pipewire-pulse`, the default on most distributions.

It doesn't poll. The sound server announces every change, so the bar updates as soon as you press a volume key.

## A volume widget

Scroll to change the volume, click to mute:

```rust
use amane::{App, Audio, Button, LayerWindow, Parent, Rectangle, Scroll, Service, Text};

fn main() {
    App::new().window(view).run();
}

fn view() -> LayerWindow {
    let audio = Audio::read();

    let label = if audio.muted() {
        String::from("muted")
    } else {
        format!("volume {}%", audio.volume())
    };

    LayerWindow::new().width(200.0).height(30.0).child(
        Rectangle::new()
            .width(Parent)
            .height(Parent)
            .on_click(clicked)
            .on_scroll(scrolled)
            .child(Text::new(label)),
    )
}

fn clicked(_: Button) {
    Audio::toggle_mute();
}

// each wheel step moves the volume by 5
fn scrolled(scroll: Scroll) {
    let volume = i32::from(Audio::read().volume());

    let step = if scroll.y < 0.0 { 5 } else { -5 };

    let changed = (volume + step).clamp(0, 100);

    Audio::set_volume(changed as u8);
}
```

## Reference

| Function | Gives or does |
|---|---|
| `volume()` | speaker volume, 0 to 100 |
| `muted()` | `true` when the speaker is muted |
| `microphone_volume()` | microphone volume, 0 to 100 |
| `microphone_muted()` | `true` when the microphone is muted |
| `Audio::set_volume(volume)` | sets the speaker volume, 0 to 100 |
| `Audio::toggle_mute()` | mutes or unmutes the speaker |
| `Audio::set_microphone_volume(volume)` | sets the microphone volume, 0 to 100 |
| `Audio::toggle_microphone_mute()` | mutes or unmutes the microphone |
| `apps()` | every app playing sound, as `AudioApp`s |
| `Audio::set_app_volume(&app, volume)` | sets every stream the app plays, 0 to 100 |
| `Audio::toggle_app_mute(&app)` | mutes the app, or unmutes it once all its streams are muted |

Each `AudioApp` has:

| Function | Gives |
|---|---|
| `name()` | what the app calls itself, like "Firefox" |
| `icon()` | its icon name, like "firefox", or `None` when it gives none |
| `binary()` | its program's file name, like "firefox", or `None` |
| `process()` | the id of the process playing the sound, or `None` |
| `volume()` | the loudest of its streams, 0 to 100 |
| `muted()` | `true` once every stream it plays is muted |

## Volume per app

`apps()` lists every app playing sound, oldest first, and each `AudioApp` can be set or muted on its own. An app often plays several streams at once, like one per browser tab, and they all move together.

Electron apps like Discord and Teams all call themselves "Chromium". `AudioApp` keeps them apart by the process playing the sound, and `process()` gives it, so you can look up which app it really is.

```rust
use amane::{App, Audio, AudioApp, Column, LayerWindow, Rectangle, Service, Text, Widget};

fn main() {
    App::new().window(view).run();
}

fn view() -> LayerWindow {
    let audio = Audio::read();

    let mut rows: Vec<Box<dyn Widget>> = Vec::new();

    for app in audio.apps() {
        rows.push(Box::new(row(app)));
    }

    LayerWindow::new().width(300.0).height(200.0).child(Column::new(rows))
}

// click to mute the app
fn row(app: &AudioApp) -> Rectangle {
    let label = format!("{} {}%", app.name(), app.volume());

    let clicked = app.clone();

    Rectangle::new()
        .width(300.0)
        .height(30.0)
        .on_click(move |_| Audio::toggle_app_mute(&clicked))
        .child(Text::new(label))
}
```

`examples/mixer.rs` adds scrolling to change each app's volume.

"Speaker" and "microphone" mean your default output and input devices. When you switch the default, for example by plugging in headphones, `Audio` follows.

The control functions run on a background thread, so a slow sound server never freezes the shell. The new value shows up when the sound server announces it, usually right away.

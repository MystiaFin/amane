# System: Battery, CPU, Memory, Brightness

Amane comes with Services for the system data most bars show. You use them like your own Services ([State and Services](services.md)): `read()` in a view, and the window redraws when the value changes. Each one starts the first time you read it.

## All four at once

```rust
use amane::{App, Battery, Brightness, Cpu, Full, LayerWindow, Memory, Service, Text};

fn main() {
    App::new().window(view).run();
}

fn view() -> LayerWindow {
    let battery = Battery::read();
    let cpu = Cpu::read();
    let memory = Memory::read();
    let brightness = Brightness::read();

    let label = format!(
        "battery {}%   cpu {}%   memory {}%   brightness {}%",
        battery.percent(),
        cpu.percent(),
        memory.percent(),
        brightness.percent()
    );

    LayerWindow::new().width(Full).height(30.0).child(Text::new(label))
}
```

`Service` has to be in the `use` line, because `read()` comes from the `Service` trait.

## Battery

Read from the kernel (`/sys/class/power_supply`), every 5 seconds.

| Function | Gives |
|---|---|
| `present()` | `false` on a machine without a battery |
| `percent()` | charge, 0 to 100 |
| `charging()` | `true` while charging |
| `full()` | plugged in and not charging, usually because it's full |

On a desktop, `present()` is `false` and everything else reads 0. Hide the battery widget there:

```rust,ignore
if battery.present() {
    // show the battery
}
```

## CPU

Read from the kernel (`/proc/stat`), every 2 seconds.

| Function | Gives |
|---|---|
| `percent()` | usage across all cores, 0 to 100 |

CPU usage only exists as a difference between two readings. The very first reading is the average since the machine booted, and from 2 seconds on it shows current usage.

## Memory

Read from the kernel (`/proc/meminfo`), every 2 seconds.

| Function | Gives |
|---|---|
| `percent()` | memory in use, 0 to 100 |
| `used_kib()` | memory in use, in kibibytes |
| `total_kib()` | all memory, in kibibytes |

"In use" means what programs hold. Cache the kernel can free right away doesn't count, the same way `free` reports it.

## Brightness

Reads the screen backlight from the kernel (`/sys/class/backlight`).

| Function | Gives |
|---|---|
| `present()` | `false` on a monitor without a backlight, like most desktop monitors |
| `percent()` | brightness, 0 to 100 |
| `Brightness::set(percent)` | sets the brightness |

`set` goes through logind, which lets the user sitting at the machine change the backlight without root and without a password. It never goes below 1%, because a black screen is hard to undo when you can't see it.

Scroll over a widget to change the brightness:

```rust
use amane::{App, Brightness, LayerWindow, Parent, Rectangle, Scroll, Service, Text};

fn main() {
    App::new().window(view).run();
}

fn view() -> LayerWindow {
    let brightness = Brightness::read();

    LayerWindow::new().width(200.0).height(30.0).child(
        Rectangle::new()
            .width(Parent)
            .height(Parent)
            .on_scroll(scrolled)
            .child(Text::new(format!("brightness {}%", brightness.percent()))),
    )
}

// each wheel step moves the brightness by 5
fn scrolled(scroll: Scroll) {
    let current = i32::from(Brightness::read().percent());

    let step = if scroll.y < 0.0 { 5 } else { -5 };

    let changed = (current + step).clamp(1, 100);

    Brightness::set(changed as u8);
}
```

`set` runs on a background thread, so a slow answer never freezes the shell.

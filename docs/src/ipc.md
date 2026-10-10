# IPC

IPC lets other programs talk to your running shell. Its main use is keybinds: your compositor runs `amane ipc call toggle-launcher`, and your shell opens the launcher.

## Toggling a window from the command line

```rust
use std::time::Duration;

use amane::{App, LayerWindow, Parent, Rectangle, Service};

struct Launcher {
    open: bool,
}

impl Service for Launcher {
    fn new() -> Self {
        Self { open: false }
    }

    fn interval() -> Duration {
        Duration::from_secs(3600)
    }
}

fn main() {
    App::new().ipc("toggle-launcher", toggle).window(view).run();
}

fn view() -> LayerWindow {
    let launcher = Launcher::read();

    LayerWindow::new()
        .width(400.0)
        .height(300.0)
        .visible(launcher.open)
        .child(Rectangle::new().width(Parent).height(Parent).fill("#1e1e2e"))
}

fn toggle(_: &[String]) -> String {
    let mut launcher = Launcher::write();

    launcher.open = !launcher.open;

    if launcher.open {
        String::from("opened")
    } else {
        String::from("closed")
    }
}
```

From a terminal:

```sh
amane ipc call toggle-launcher
```

It prints `opened`, and the launcher appears. Run it again to close it.

## How a call reaches your shell

- `App::ipc(name, handler)` registers a handler under a name. Call it once per name.
- Your shell listens on a socket, `$XDG_RUNTIME_DIR/amane.sock`.
- `amane ipc call <name> [arguments...]` connects to that socket, sends the name and arguments, and prints the reply.
- The handler gets the arguments and returns a `String`. Whatever it returns is printed by `amane ipc call`.
- The handler writes to a Service, so every window that reads it redraws ([State and Services](services.md)).

The handler runs on the same thread that draws, so keep it quick. For slow work, start a thread or use `amane::spawn`.

## Passing arguments

Everything after the name is passed to the handler as a list of strings:

```rust,ignore
fn main() {
    App::new().ipc("say", say).window(view).run();
}

fn say(arguments: &[String]) -> String {
    let words = arguments.join(" ");

    Message::write().text = words.clone();

    format!("showing: {words}")
}
```

```sh
amane ipc call say hello world
```

The handler gets `["hello", "world"]`. Arguments can't contain newlines.

Calling a name that has no handler answers `no handler named <name>`.

## Binding calls to keys

niri:

```kdl
binds {
    Mod+Space { spawn "amane" "ipc" "call" "toggle-launcher"; }
}
```

Hyprland:

```text
bind = SUPER, SPACE, exec, amane ipc call toggle-launcher
```

Sway:

```text
bindsym $mod+space exec amane ipc call toggle-launcher
```

## One shell at a time

There's one IPC socket per session, so only one Amane app can listen for calls at a time. Starting a second listener stops it right away with `failed to listen: amane is already running`. `amane dev` restarts its own shell without hitting this, but it can't stop a shell started some other way. Stop your everyday shell (the one from `amane run`) before running `amane dev`.

For a standalone window that does not need IPC, use `App::new().without_ipc()`. It can run alongside your shell and leaves the shell's socket alone. IPC is enabled by default; `without_ipc()` disables the listener even if handlers have been registered.

If `amane ipc call` says `amane is not running`, no shell is listening for IPC in this session.

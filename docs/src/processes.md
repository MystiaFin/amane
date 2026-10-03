# Commands and Files

This page covers running other programs and watching files: the glue for anything Amane doesn't read for you.

| Function | Does | Waits? |
|---|---|---|
| `amane::spawn(command)` | starts a command and moves on | no |
| `amane::output(command)` | runs a command and returns what it printed | yes, until it exits |
| `amane::lines(command)` | runs a long-running command and gives each line it prints | yes, for each line |
| `amane::watch_file(path)` | gives one item every time a file changes | yes, for each change |

All commands run through `sh -c`, so pipes, globs, `~`, and `&&` work like in a terminal.

## Starting programs

```rust,ignore
.on_click(|_| amane::spawn("firefox"))
.on_click(|_| amane::spawn("niri msg action focus-workspace-down"))
.on_click(|_| amane::spawn("notify-send amane 'the bar was clicked'"))
```

`spawn` returns right away, so it's safe in input handlers.

## Reading a command's output

`output` waits for the command to finish, so never call it in a view, where it would freeze every window. Call it in a Service, in `new()` or `update()`, which run on the Service's own thread:

```rust
use std::time::Duration;

use amane::{App, Full, LayerWindow, Service, Text};

struct Kernel {
    version: String,
}

impl Service for Kernel {
    fn new() -> Self {
        Self { version: amane::output("uname -r") }
    }

    // never changes while the shell runs
    fn interval() -> Duration {
        Duration::from_secs(3600)
    }
}

fn main() {
    App::new().window(view).run();
}

fn view() -> LayerWindow {
    let kernel = Kernel::read();

    LayerWindow::new().width(Full).height(30.0).child(Text::new(&kernel.version))
}
```

`output` trims the trailing newline. A command that fails or prints nothing gives an empty string.

## Following a command's output

Some commands print a line every time something happens: `niri msg event-stream`, `pactl subscribe`, `udevadm monitor`. `lines` gives each line as it comes, in a Service's `listen`:

```rust
use amane::{App, Full, LayerWindow, Service, Text};

struct Events {
    last: String,
}

impl Service for Events {
    fn new() -> Self {
        Self { last: String::from("waiting") }
    }

    fn listen() {
        for line in amane::lines("niri msg event-stream") {
            Self::write().last = line;
        }
    }
}

fn main() {
    App::new().window(view).run();
}

fn view() -> LayerWindow {
    let events = Events::read();

    LayerWindow::new().width(Full).height(30.0).child(Text::new(&events.last))
}
```

The loop ends when the command exits. If the loop stops early, for example because `listen` panicked, the command is killed, so nothing is left running.

## Watching a file

```rust
use std::fs;

use amane::{App, Full, LayerWindow, Service, Text};

const NOTE: &str = "/tmp/amane-note";

struct Note {
    text: String,
}

impl Service for Note {
    fn new() -> Self {
        Self { text: read_note() }
    }

    fn listen() {
        for _ in amane::watch_file(NOTE) {
            Self::write().text = read_note();
        }
    }
}

fn read_note() -> String {
    fs::read_to_string(NOTE).unwrap_or_default().trim_end().to_string()
}

fn main() {
    App::new().window(view).run();
}

fn view() -> LayerWindow {
    let note = Note::read();

    LayerWindow::new().width(Full).height(30.0).child(Text::new(&note.text))
}
```

`std::fs` is Rust's standard library.

Try `echo hello > /tmp/amane-note` while the bar runs.

`watch_file` gives an item each time the file is written, created, or replaced. Editors usually save by writing a new file and renaming it over the old one, which would end a watch on the file itself, so Amane watches the folder and picks out your file's changes. The file doesn't have to exist yet when you start watching, but its folder does.

Reading the file is up to you, in the loop, as above.

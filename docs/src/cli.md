# The amane Command

| Command | What it does |
|---|---|
| `amane startup` | Creates `~/.config/amane/src/main.rs` from a template. Never overwrites an existing file. |
| `amane startup --example` | Creates a full example bar in `~/.config/amane/src/`, split over several files, with workspace buttons for each monitor. Never overwrites an existing file. |
| `amane dev` | Builds your shell, starts it, and rebuilds and restarts it on every save. |
| `amane compile` | Builds the optimized shell without starting it. |
| `amane run` | Builds the optimized shell if anything changed, then starts it. |
| `amane ipc call <name> [arguments...]` | Calls a handler in the running shell. See [IPC](ipc.md). |

## Where things live

- **Your shell:** `~/.config/amane/src/`. `main.rs` is the entry point. You can add more files next to it and load them with `mod`, like in any Rust program.
- **Build files:** `~/.cache/amane/`.
  - `library/` holds the copy of Amane unpacked from the command.
  - `project/` holds the Cargo project generated around your `main.rs`.
  - `project/target/` holds the build output.

Both paths follow `XDG_CONFIG_HOME` and `XDG_CACHE_HOME` when they're set.

You never edit anything in the cache folder. Deleting it is safe. The next build makes it again, from scratch.

## Splitting your shell into files

`main.rs` can load other files the usual Rust way:

```text
~/.config/amane/src/
├── main.rs
├── bar.rs
└── clock.rs
```

```rust,ignore
// main.rs
mod bar;
mod clock;

use amane::App;

fn main() {
    App::new().window(bar::view).run();
}
```

`amane dev` watches the whole `src/` folder, so saving any of these files triggers a rebuild.

## Dev builds and release builds

| | `amane dev` | `amane run`, `amane compile` |
|---|---|---|
| Your code | not optimized, builds in seconds | optimized |
| Amane and its dependencies | optimized | optimized |
| Use it for | writing your shell | daily use |

Amane is optimized even in dev builds, because unoptimized drawing is too slow for smooth animation. Only your own code is left unoptimized, which keeps rebuilds after a save quick.

## Dependencies

Your shell's `Cargo.toml` is generated for you on every build, and it only depends on `amane`. That means you can't add other crates to your shell yet.

Most of what a shell needs is in the library already:

- system data, through [Built-in Services](system.md)
- running commands and reading their output, through [Commands and Files](processes.md)
- file watching, D-Bus, and IPC

For anything else, like the current time, run a command with `amane::output` (for example `date +%H:%M`).

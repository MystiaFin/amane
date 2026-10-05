# The amane Command

| Command | What it does |
|---|---|
| `amane startup` | Creates `~/.config/amane/src/main.rs` from a template, plus a `Cargo.toml` for your editor. Never overwrites an existing shell file. |
| `amane startup --example` | Creates a full example bar in `~/.config/amane/src/`, split over several files, with workspace buttons for each monitor. Never overwrites an existing file. |
| `amane dev` | Builds your shell, starts it, and rebuilds and restarts it on every save. |
| `amane compile` | Builds your shell and saves it as `~/.cache/amane/amane-shell`, without starting it. |
| `amane run` | Starts the shell `amane compile` saved. Never builds. |
| `amane clean` | Deletes the build output in `~/.cache/amane/project/`. Keeps the compiled shell, so `amane run` still works. |
| `amane ipc call <name> [arguments...]` | Calls a handler in the running shell. See [IPC](ipc.md). |

## Where things live

- **Your shell:** `~/.config/amane/src/`. `main.rs` is the entry point. You can add more files next to it and load them with `mod`, like in any Rust program.
- **Editor support:** `~/.config/amane/Cargo.toml`. `amane startup` writes it and every build refreshes it, so rust-analyzer finds Amane and completes its types in any editor. Don't edit it. It holds paths for this machine only, so if you keep your shell in git, ignore `Cargo.toml`, `Cargo.lock` and `target/`.
- **Build files:** `~/.cache/amane/`.
  - `library/` holds the copy of Amane unpacked from the command.
  - `project/` holds the Cargo project generated around your `main.rs`.
  - `project/target/` holds the build output.
  - `amane-shell` is the shell `amane compile` saved, the one `amane run` starts.

Both paths follow `XDG_CONFIG_HOME` and `XDG_CACHE_HOME` when they're set.

You never edit anything in the cache folder. Deleting it is safe. The next build makes it again, from scratch.

`project/` is the big one, often close to 1 GB. It only makes rebuilds fast, and your shell never reads it. Run `amane clean` to delete it. The next `amane compile` or `amane dev` then builds everything again from scratch, which takes a few minutes. To build and shrink in one go:

```sh
amane compile && amane clean
```

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

## How your shell is built

`amane dev` and `amane compile` share one build, so the build output is only kept on disk once.

| | Optimized |
|---|---|
| Your code | no, so a save rebuilds in seconds |
| Amane and its dependencies | yes |

Amane is always optimized, because unoptimized drawing is too slow for smooth animation. Your own code only describes the shell and Amane does the heavy work, so leaving your code unoptimized keeps rebuilds quick without slowing the shell down.

## Dependencies

Your shell's `Cargo.toml` is generated for you on every build, and it only depends on `amane`. That means you can't add other crates to your shell yet.

Most of what a shell needs is in the library already:

- system data, through [Built-in Services](system.md)
- running commands and reading their output, through [Commands and Files](processes.md)
- file watching, D-Bus, and IPC

For anything else, like the current time, run a command with `amane::output` (for example `date +%H:%M`).

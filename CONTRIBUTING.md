# Contributing

Thanks for helping with Amane. It's a small project, so clear issues and small pull requests make the biggest difference.

## Opening an issue

Pick a template when you open an issue. Blank issues are turned off, because an issue without details can't be worked on.

- **One request per issue.** "Workspace indicator for Umbriel" and "keyboard layout indicator" are two issues, even if you want both.
- **Say exactly what you want.** Name the widget or service, what it should show, and what you'd do with it in your shell.
- **Search first.** Your request may already be open, or listed in the pinned roadmap issue.

### Compositor support

Amane has two levels of compositor support, so "add support for compositor X" can mean either one:

- **Showing windows** works on any compositor that supports wlr-layer-shell. If your compositor does, Amane already runs there. GNOME doesn't support it.
- **Compositor-specific services**, like workspaces, need code for each compositor's own IPC. Right now that's niri, Hyprland and Sway (`src/compositor/`).

For a compositor-specific request, say which service you need and link the compositor's IPC documentation.

## Pull requests

- Open an issue first for anything bigger than a small fix, so we can agree on the design before you write it.
- Read [ARCHITECTURE.md](ARCHITECTURE.md) before changing the library. It explains the rules the code depends on.
- Keep each pull request to one change, and match the style of the code around it.
- Update the docs in `docs/src/` when you change something users can see.

### Building and testing

With Nix, `nix develop` gives you every library Amane links. Without Nix, `./install.sh` installs them (see the [README](README.md)).

The tests CI runs don't need a compositor:

```sh
cargo test --lib
```

For anything that draws, also run an example on a real compositor, for example `cargo run --example bar`.

### Commit messages

Short, lowercase, and say what the change does: `add sway workspace support`, `keep named hyprland workspaces`.

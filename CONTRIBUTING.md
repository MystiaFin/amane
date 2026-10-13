# Contributing to Amane

Thanks for your interest in Amane.
It's a small project, so please keep issues and pull requests small and clear.

## LLM Rules

You can use LLMs to generate code, but don't let an LLM decide what things should be.
Please be thoughtful with your design decisions.

Never post a PR with code you don't understand.

I know I can't enforce this, so I'm trusting you to follow it.

## Opening an issue

Please pick a template when you open an issue.
Blank issues are turned off, because I can't work on an issue without details.

- Keep it to one request per issue. "Workspace indicator for Umbriel" and "keyboard layout indicator" are two issues, even if you want both.
- Say exactly what you want: which widget or service, what it should show, and how you'd use it in your shell.
- Search first. Your request may already be open, or listed in the pinned roadmap issue.

### Compositor support

"Add support for compositor X" can mean two different things.

Showing windows works on any compositor that supports wlr-layer-shell.
If yours does, Amane already runs there.
GNOME doesn't support it.

Compositor-specific services, like workspaces, need code for each compositor's own IPC.
Right now that's niri, Hyprland, Sway and Mango (`src/compositor/`).
For these, say which service you need and link the compositor's IPC documentation.

## Pull requests

- Open an issue first for anything bigger than a small fix, so we can agree on the design before you write the code.
- Read [ARCHITECTURE.md](ARCHITECTURE.md) before changing the library. It explains the rules the code depends on.
- Keep each pull request to one change, and match the style of the surrounding code.
- Update the docs in `docs/src/` when you change something users can see.

### Building and testing

With Nix, `nix develop` gives you every library Amane links.
Without Nix, `./install.sh` installs them (see the [README](README.md)).

The tests that CI runs don't need a compositor:

```sh
cargo test --lib
```

For anything that draws, also run an example on a real compositor, like `cargo run --example bar`.

### Commit messages

Keep them short and lowercase, and say what the change does: `add sway workspace support`, `keep named hyprland workspaces`.

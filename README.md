<p align="center">
  <img src="amane.svg" alt="amane logo" width="128">
</p>

<h1 align="center">amane</h1>

<p align="center">
  <img src="https://img.shields.io/badge/version-0.1.1-de7979" alt="version 0.1.1">
  <img src="https://img.shields.io/badge/status-experimental-7c1f1f" alt="status: experimental">
  <img src="https://img.shields.io/badge/rust-2024_edition-16151d?logo=rust&logoColor=white" alt="rust 2024 edition">
</p>

Amane is a Rust library for building Wayland desktop shells, like bars, panels and launchers. You write your shell in plain Rust, and the `amane` CLI builds and runs it for you.

Keep in mind, Amane is still early (0.1) and experimental, so things will break and change.

## Documentation

The full guide is at [mystiafin.github.io/amane](https://mystiafin.github.io/amane/). It walks you from your first shell through layout, widgets, input and animation, plus the built-in services for audio, network, bluetooth, notifications and workspaces.

## Installation

Before installing, check that you have what a shell needs to run:

- a Wayland compositor that supports wlr-layer-shell, for example niri, Hyprland or Sway. GNOME doesn't support it, so Amane won't work there.

### Manual installation

#### 1. Install Rust

You need a Rust toolchain with cargo. Cargo has to stay on your `PATH` after the install, because `amane dev` and `amane compile` call cargo to build your config.

#### 2. Run the install script

Clone the repo and run `install.sh`. It installs the system libraries Amane needs with pacman, apt or dnf, then installs the CLI with `cargo install --path cli`:

```sh
git clone https://github.com/MystiaFin/amane.git
cd amane
./install.sh
```

If your distro uses another package manager, the script stops and lists what to install yourself: a C compiler, pkg-config, wayland, libxkbcommon, fontconfig, freetype, expat, vulkan-loader, libpulseaudio, linux-pam, polkit, GLib and GObject, with their headers. After that, run `cargo install --path cli`.

This puts `amane` in `~/.cargo/bin`, so make sure that folder is on your `PATH` too.

### NixOS (flake)

Add amane to your flake inputs, then add its package to your system packages:

```nix
inputs.amane.url = "github:MystiaFin/amane";
```

```nix
environment.systemPackages = [ inputs.amane.packages.x86_64-linux.default ];
```

or install it into your profile:

```sh
nix profile install github:MystiaFin/amane
```

## Commands

Your shell lives in `~/.config/amane/src/main.rs`. If you set `XDG_CONFIG_HOME`, Amane uses `$XDG_CONFIG_HOME/amane` instead.

### amane startup

Creates `~/.config/amane/src/main.rs` with a small starter shell. If that file already exists, it stops with an error, so it never overwrites your shell.

```sh
amane startup
```

For a full example bar instead, split over several files with workspace buttons for each monitor, add `--example`. It also stops if any of its files already exist.

```sh
amane startup --example
```

### amane dev

Builds your shell, starts it, and then rebuilds and restarts it every time you save. If a build fails, the old shell keeps running while you fix the error.

```sh
amane dev
```

### amane compile

Builds your shell and saves it as `~/.cache/amane/amane-shell`, without starting it.

```sh
amane compile
```

### amane run

Starts the shell `amane compile` saved. It never builds, so after editing your shell, run `amane compile` first.

```sh
amane run
```

### amane clean

Deletes the build output, which is often close to 1 GB. The compiled shell stays, so `amane run` still works. The next `amane compile` or `amane dev` builds everything again from scratch, so it asks first. `--yes` skips the question.

```sh
amane clean
amane clean --yes
```

## Example

This is the starter config that `amane startup` creates. It puts a 30 pixel blue bar at the top of the screen with some text in it.

```rust
use amane::{App, Color, Full, Layer, LayerWindow, Parent, Rectangle, Text, Vertical};

fn main() {
    App::new().window(view).run();
}

fn view() -> LayerWindow {
    LayerWindow::new()
        .width(Full)
        .height(30.0)
        .anchor_vertical(Vertical::Top)
        .layer(Layer::Top)
        .child(
            Rectangle::new()
                .width(Parent)
                .height(Parent)
                .fill(Color::BLUE)
                .child(Text::new("hello world").size(20.0).color(Color::WHITE)),
        )
}
```

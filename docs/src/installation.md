# Installation

Amane comes as one command, `amane`. The command carries a full copy of the library inside it. It builds your shell from your config folder, so you never set up a Cargo project yourself.

## NixOS

The repository is a flake. Its package wraps `amane` with everything it needs at build time (cargo, rustc, pkg-config, and the system libraries), so there's nothing else to install.

To try it without installing:

```sh
nix run github:MystiaFin/amane -- startup
```

To install it for your user:

```sh
nix profile install github:MystiaFin/amane
```

To install it system-wide, add it to your flake inputs:

```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    amane.url = "github:MystiaFin/amane";
  };

  outputs = { nixpkgs, amane, ... }: {
    nixosConfigurations.your-host = nixpkgs.lib.nixosSystem {
      system = "x86_64-linux";
      modules = [
        ./configuration.nix
        {
          environment.systemPackages = [ amane.packages.x86_64-linux.default ];
        }
      ];
    };
  };
}
```

Replace `your-host` with your host's name, then rebuild.

## Other distributions

You need a Rust toolchain, `pkg-config`, and these libraries with their development headers:

| Library | Debian / Ubuntu | Arch |
|---|---|---|
| Wayland | `libwayland-dev` | `wayland` |
| xkbcommon | `libxkbcommon-dev` | `libxkbcommon` |
| fontconfig | `libfontconfig1-dev` | `fontconfig` |
| Vulkan loader | `libvulkan1` | `vulkan-icd-loader` |
| PulseAudio client | `libpulse-dev` | `libpulse` |
| PAM | `libpam0g-dev` | `pam` |
| Polkit agent | `libpolkit-agent-1-dev` | `polkit` |
| GLib / GObject | `libglib2.0-dev` | `glib2` |

Then build and install the command:

```sh
git clone https://github.com/MystiaFin/amane
cd amane
cargo install --path cli
```

`cargo install` puts `amane` in `~/.cargo/bin`. Make sure that folder is on your `PATH`.

The `amane` command runs `cargo` every time it builds your shell, so keep the Rust toolchain installed.

## Updating

The library is copied into the `amane` command when the command is built. To get a newer Amane, update the command itself:

- NixOS: `nix profile upgrade amane`, or update the flake input and rebuild.
- Other distributions: `git pull` in the clone, then run `cargo install --path cli` again.

The next `amane compile` or `amane dev` rebuilds your shell with the new library.

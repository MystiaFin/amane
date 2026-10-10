{
  description = "Minimal Rust Wayland shell experiment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs =
    { nixpkgs, ... }:
    let
      system = "x86_64-linux";

      pkgs = import nixpkgs {
        inherit system;
      };

      # amane links these, both when the cli is built and when it builds a shell
      libraries = with pkgs; [
        fontconfig
        freetype
        expat
        wayland
        libxkbcommon
        vulkan-loader
        libpulseaudio
        linux-pam
        polkit
        glib
      ];
    in
    {
      packages.${system}.default = pkgs.rustPlatform.buildRustPackage {
        pname = "amane";
        version = "0.1.1";

        src = ./.;

        cargoLock.lockFile = ./Cargo.lock;

        cargoBuildFlags = [ "-p" "amane-cli" ];

        # the workspace tests need a running compositor
        doCheck = false;

        nativeBuildInputs = with pkgs; [
          pkg-config
          makeWrapper
        ];

        buildInputs = libraries;

        # the cli runs cargo on the user's config, so it carries its own toolchain;
        # the native authentication libraries are linked by name, so they need a search path
        postFixup = with pkgs; ''
          wrapProgram $out/bin/amane \
            --prefix PATH : ${lib.makeBinPath [ cargo rustc pkg-config stdenv.cc ]} \
            --prefix PKG_CONFIG_PATH : ${lib.makeSearchPathOutput "dev" "lib/pkgconfig" libraries} \
            --prefix LIBRARY_PATH : ${lib.makeLibraryPath [ linux-pam polkit glib ]} \
            --prefix LD_LIBRARY_PATH : ${lib.makeLibraryPath [ vulkan-loader wayland ]}
        '';
      };

      devShells.${system}.default = pkgs.mkShell {
        nativeBuildInputs = with pkgs; [
          rustc
          cargo
          rustfmt
          clippy
          rust-analyzer

          pkg-config
          fontconfig
        ];

        buildInputs = with pkgs; [
          wayland
          wayland-protocols
          libxkbcommon
          vulkan-loader
          libpulseaudio
          linux-pam
          polkit
          glib
        ];

        LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [
          pkgs.vulkan-loader
          pkgs.wayland
        ];

        packages = with pkgs; [
          wayland-utils
          dbus
        ];
      };
    };
}

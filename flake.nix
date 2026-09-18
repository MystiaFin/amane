{
  description = "Minimal Rust Wayland shell experiment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = { nixpkgs, ... }:
    let
      system = "x86_64-linux";

      pkgs = import nixpkgs {
        inherit system;
      };
    in
    {
      devShells.${system}.default = pkgs.mkShell {
        nativeBuildInputs = with pkgs; [
          rustc
          cargo
          rustfmt
          clippy
          rust-analyzer

          pkg-config
        ];

        buildInputs = with pkgs; [
          wayland
          wayland-protocols
          libxkbcommon
        ];

        packages = with pkgs; [
          wayland-utils
        ];
      };
    };
}


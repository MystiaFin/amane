#!/bin/sh
# installs the system libraries amane links, then the amane cli
set -eu

# checked first, so nothing is installed when the build can't run anyway
if ! command -v cargo > /dev/null; then
    echo "failed to find cargo, install rust from https://rustup.rs and run this again"
    exit 1
fi

# the libraries come from the same list as flake.nix
if command -v pacman > /dev/null; then
    sudo pacman -S --needed gcc pkgconf wayland libxkbcommon fontconfig freetype2 expat \
        vulkan-icd-loader libpulse pam
elif command -v apt-get > /dev/null; then
    sudo apt-get install -y build-essential pkg-config libwayland-dev libxkbcommon-dev \
        libfontconfig-dev libfreetype-dev libexpat1-dev libvulkan1 libpulse-dev libpam0g-dev
elif command -v dnf > /dev/null; then
    sudo dnf install -y gcc pkgconf-pkg-config wayland-devel libxkbcommon-devel fontconfig-devel \
        freetype-devel expat-devel vulkan-loader pulseaudio-libs-devel pam-devel
else
    echo "failed to find pacman, apt-get or dnf, install these yourself:"
    echo "  a c compiler, pkg-config, wayland, libxkbcommon, fontconfig, freetype,"
    echo "  expat, vulkan-loader, libpulseaudio, linux-pam (with their headers)"
    exit 1
fi

# the script can be run from any folder, the cli is next to it
cd "$(dirname "$0")"

cargo install --path cli

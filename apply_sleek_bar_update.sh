#!/usr/bin/env bash
set -euo pipefail

if [ ! -f "Cargo.toml" ] || [ ! -d "tools/dotfiles-center" ]; then
  echo "Run this from the root of your SplinterDots repo."
  exit 1
fi

backup_dir=".dotfiles-change-backup/$(date +%Y%m%d-%H%M%S)"
mkdir -p "$backup_dir/tools/dotfiles-center/src" "$backup_dir/packages"

if [ -f tools/dotfiles-center/src/main.rs ]; then
  cp -a tools/dotfiles-center/src/main.rs "$backup_dir/tools/dotfiles-center/src/main.rs"
fi

if [ -f packages/arch.txt ]; then
  cp -a packages/arch.txt "$backup_dir/packages/arch.txt"
fi

cp files/tools/dotfiles-center/src/main.rs tools/dotfiles-center/src/main.rs

# Make sure useful widget/icon packages are present in the package list.
if [ -f packages/arch.txt ]; then
  add_pkg() {
    local pkg="$1"
    grep -qxF "$pkg" packages/arch.txt || printf '%s\n' "$pkg" >> packages/arch.txt
  }

  add_pkg rust
  add_pkg cargo
  add_pkg playerctl
  add_pkg brightnessctl
  add_pkg bluez-utils
  add_pkg lm_sensors
  add_pkg pacman-contrib
  add_pkg ttf-nerd-fonts-symbols
  add_pkg otf-font-awesome
fi

cargo build --release --manifest-path Cargo.toml -p dotfiles-center

echo
echo "Done."
echo "Backups are in: $backup_dir"
echo
echo "Run:"
echo "  ./install.sh --dry-run"
echo "  ./install.sh --packages"
echo "  ./install.sh"
echo "  dotctl center"
echo
echo "Inside Dotfiles Center:"
echo "  1. Open QML Bar and set icon pack/font/speed."
echo "  2. Open Widgets and enable/configure widgets."
echo "  3. Press Save changes."
echo "  4. Press Save and restart bar."

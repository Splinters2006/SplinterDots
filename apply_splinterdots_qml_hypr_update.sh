#!/usr/bin/env bash
set -euo pipefail

if [ ! -f "install.sh" ] || [ ! -d "home" ] || [ ! -d "scripts" ]; then
  echo "Run this script from the root of your SplinterDots repository."
  exit 1
fi

backup_dir=".dotfiles-change-backup/$(date +%Y%m%d-%H%M%S)"
mkdir -p "$backup_dir"

copy_file() {
  local src="$1"
  local dest="$2"

  if [ -e "$dest" ] || [ -L "$dest" ]; then
    mkdir -p "$backup_dir/$(dirname "$dest")"
    cp -a "$dest" "$backup_dir/$dest"
  fi

  mkdir -p "$(dirname "$dest")"
  cp "$src" "$dest"
  echo "updated $dest"
}

copy_file "files/scripts/dotfiles-center" "scripts/dotfiles-center"
copy_file "files/config/dotfiles/hyprland-options.json" "config/dotfiles/hyprland-options.json"
copy_file "files/home/.config/hypr/dotfiles-generated.conf" "home/.config/hypr/dotfiles-generated.conf"
copy_file "files/home/.config/hypr/hyprland.conf" "home/.config/hypr/hyprland.conf"
copy_file "files/home/.config/hypr/keybindings.conf" "home/.config/hypr/keybindings.conf"
copy_file "files/config/dotfiles/settings.conf" "config/dotfiles/settings.conf"
copy_file "files/packages/arch.txt" "packages/arch.txt"
copy_file "files/scripts/dotfiles-hypr-autostart" "scripts/dotfiles-hypr-autostart"
copy_file "files/home/.config/quickshell/splinterbar/shell.qml" "home/.config/quickshell/splinterbar/shell.qml"
copy_file "files/README.md" "README.md"

rm -rf "home/.config/waybar"

chmod +x scripts/dotfiles-center
chmod +x scripts/dotfiles-hypr-autostart

echo
echo "Done."
echo "Backups are in: $backup_dir"
echo
echo "Next commands:"
echo "  git status"
echo "  ./install.sh --dry-run"
echo "  ./install.sh --packages"
echo "  ./install.sh"
echo
echo "Then open Dotfiles Center:"
echo "  dotctl center"

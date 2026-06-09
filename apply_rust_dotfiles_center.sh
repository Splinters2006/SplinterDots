#!/usr/bin/env bash
set -euo pipefail

if [ ! -f "install.sh" ] || [ ! -d "scripts" ]; then
  echo "Run this from the root of your SplinterDots repo."
  exit 1
fi

backup_dir=".dotfiles-change-backup/$(date +%Y%m%d-%H%M%S)"
mkdir -p "$backup_dir"

backup_path() {
  local path="$1"
  if [ -e "$path" ] || [ -L "$path" ]; then
    mkdir -p "$backup_dir/$(dirname "$path")"
    cp -a "$path" "$backup_dir/$path"
  fi
}

add_package_once() {
  local package_file="$1"
  local package_name="$2"

  touch "$package_file"

  if ! grep -qxF "$package_name" "$package_file"; then
    printf '%s\n' "$package_name" >> "$package_file"
  fi
}

remove_package() {
  local package_file="$1"
  local package_name="$2"
  local temp_file

  [ -f "$package_file" ] || return 0
  temp_file="$(mktemp)"
  grep -vxF "$package_name" "$package_file" > "$temp_file" || true
  cat "$temp_file" > "$package_file"
  rm -f "$temp_file"
}

backup_path "scripts/dotfiles-center"
backup_path "packages/arch.txt"
backup_path "tools/dotfiles-center-rs"

mkdir -p tools/dotfiles-center-rs/src
cp files/tools/dotfiles-center-rs/Cargo.toml tools/dotfiles-center-rs/Cargo.toml
cp files/tools/dotfiles-center-rs/src/main.rs tools/dotfiles-center-rs/src/main.rs
cp files/scripts/dotfiles-center scripts/dotfiles-center
chmod +x scripts/dotfiles-center

if [ -f packages/arch.txt ]; then
  remove_package packages/arch.txt tk
  add_package_once packages/arch.txt rust
fi

echo
echo "Rust Dotfiles Center has been added."
echo "Backups are in: $backup_dir"
echo
echo "Next:"
echo "  ./install.sh --dry-run"
echo "  ./install.sh --packages"
echo "  ./install.sh"
echo "  dotctl center"
echo
echo "First launch can take a while because Cargo has to build the Rust app."

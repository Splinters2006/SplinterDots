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

backup_path "Cargo.toml"
backup_path "tools/dotfiles-center"
backup_path "tools/dotfiles-center-rs"
backup_path "scripts/dotfiles-center"
backup_path "packages/arch.txt"
backup_path ".gitignore"

mkdir -p tools/dotfiles-center/src
cp files/Cargo.toml Cargo.toml
cp files/tools/dotfiles-center/Cargo.toml tools/dotfiles-center/Cargo.toml
cp files/tools/dotfiles-center/src/main.rs tools/dotfiles-center/src/main.rs
cp files/scripts/dotfiles-center scripts/dotfiles-center
chmod +x scripts/dotfiles-center

# Remove the older non-workspace Rust attempt if it exists.
rm -rf tools/dotfiles-center-rs

# Update Arch package dependencies without Python.
if [ -f packages/arch.txt ]; then
  tmp="$(mktemp)"
  grep -vx 'tk' packages/arch.txt > "$tmp" || true
  mv "$tmp" packages/arch.txt

  grep -qx 'rust' packages/arch.txt || cat >> packages/arch.txt <<'PKGEOF'

# Rust build tooling
rust
PKGEOF
  grep -qx 'cargo' packages/arch.txt || echo 'cargo' >> packages/arch.txt
fi

# Do not commit Cargo build output.
touch .gitignore
grep -qx '/target/' .gitignore || echo '/target/' >> .gitignore
grep -qx '.dotfiles-change-backup/' .gitignore || echo '.dotfiles-change-backup/' >> .gitignore

echo
echo "Done. Dotfiles Center is now a Cargo workspace project."
echo "Backups are in: $backup_dir"
echo
echo "Next:"
echo "  ./install.sh --dry-run"
echo "  ./install.sh --packages"
echo "  ./install.sh"
echo "  dotctl center"
echo
echo "First launch builds the Rust app with Cargo."

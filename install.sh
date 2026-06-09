#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
HOME_DIR="${HOME:?HOME is required}"
BACKUP_DIR="$ROOT_DIR/.dotfiles-backup/$(date +%Y%m%d-%H%M%S)"
DRY_RUN=0
INSTALL_PACKAGES=0
UPDATE_DOTFILES=0
CONFIGURE_SYSTEM=0
DOTFILES_NAME="Arch User"
DOTFILES_EMAIL=""

usage() {
  cat <<'EOF'
Usage: ./install.sh [options]

Options:
  --dry-run       Show actions without changing files
  --packages      Install recommended Arch packages with pacman
  --upd           Update this dotfiles repo, then apply dotfiles
  --system        Configure greetd and enable desktop services
  --all           Update repo, install packages, configure system, and apply dotfiles
  -h, --help      Show this help
EOF
}

log() {
  printf '%s\n' "$*"
}

action_log() {
  if [ "$DRY_RUN" -eq 1 ]; then
    printf 'Would %s\n' "$*"
  else
    printf '%s\n' "$*"
  fi
}

run() {
  if [ "$DRY_RUN" -eq 1 ]; then
    printf '[dry-run] %s\n' "$*"
  else
    "$@"
  fi
}

while [ "$#" -gt 0 ]; do
  case "$1" in
    --dry-run)
      DRY_RUN=1
      ;;
    --packages)
      INSTALL_PACKAGES=1
      ;;
    --upd)
      UPDATE_DOTFILES=1
      ;;
    --system)
      CONFIGURE_SYSTEM=1
      ;;
    --all)
      UPDATE_DOTFILES=1
      INSTALL_PACKAGES=1
      CONFIGURE_SYSTEM=1
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      printf 'Unknown option: %s\n\n' "$1" >&2
      usage >&2
      exit 1
      ;;
  esac
  shift
done

if [ "$(uname -s)" != "Linux" ]; then
  log "This installer is intended for Linux. Continuing anyway."
fi

if [ -f /etc/arch-release ]; then
  IS_ARCH=1
else
  IS_ARCH=0
fi

install_packages() {
  if [ "$IS_ARCH" -ne 1 ]; then
    log "Skipping packages: /etc/arch-release was not found."
    return
  fi

  if ! command -v pacman >/dev/null 2>&1; then
    log "Skipping packages: pacman was not found."
    return
  fi

  mapfile -t packages < <(sed '/^[[:space:]]*#/d; /^[[:space:]]*$/d' "$ROOT_DIR/packages/arch.txt")

  if [ "$DRY_RUN" -eq 1 ]; then
    log "[dry-run] sudo pacman -S --needed ${packages[*]}"
    return
  fi

  sudo pacman -S --needed "${packages[@]}"
}

configure_system() {
  if ! command -v systemctl >/dev/null 2>&1; then
    log "Skipping system setup: systemctl was not found."
    return
  fi

  if [ "$DRY_RUN" -eq 1 ]; then
    log "[dry-run] sudo install -Dm644 $ROOT_DIR/system/greetd/config.toml /etc/greetd/config.toml"
    log "[dry-run] sudo systemctl enable NetworkManager greetd"
    return
  fi

  sudo install -Dm644 "$ROOT_DIR/system/greetd/config.toml" /etc/greetd/config.toml
  sudo systemctl enable NetworkManager greetd
}

update_dotfiles() {
  if ! command -v git >/dev/null 2>&1; then
    log "Skipping update: git was not found."
    return
  fi

  if ! git -C "$ROOT_DIR" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    log "Skipping update: $ROOT_DIR is not a Git worktree."
    return
  fi

  if ! git -C "$ROOT_DIR" remote get-url origin >/dev/null 2>&1; then
    log "Skipping update: no origin remote is configured."
    return
  fi

  if [ "$DRY_RUN" -eq 1 ]; then
    log "[dry-run] git -C $ROOT_DIR pull --ff-only"
    return
  fi

  git -C "$ROOT_DIR" pull --ff-only
}

load_settings() {
  if [ -f "$ROOT_DIR/config/dotfiles/settings.conf" ]; then
    # shellcheck disable=SC1091
    . "$ROOT_DIR/config/dotfiles/settings.conf"
  fi

  if [ -f "$HOME_DIR/.config/dotfiles/local.conf" ]; then
    # shellcheck disable=SC1091
    . "$HOME_DIR/.config/dotfiles/local.conf"
  fi
}

write_git_user_config() {
  local dest="$HOME_DIR/.config/dotfiles/git-user.inc"

  run mkdir -p "$(dirname "$dest")"

  if [ "$DRY_RUN" -eq 1 ]; then
    log "[dry-run] write $dest"
    return
  fi

  {
    printf '[user]\n'
    printf '\tname = %s\n' "${DOTFILES_NAME:-Arch User}"
    if [ -n "${DOTFILES_EMAIL:-}" ]; then
      printf '\temail = %s\n' "$DOTFILES_EMAIL"
    fi
  } > "$dest"
  log "wrote: $dest"
}

link_file() {
  local src="$1"
  local dest="$2"
  local dest_dir
  dest_dir="$(dirname "$dest")"

  run mkdir -p "$dest_dir"

  if [ -L "$dest" ] && [ "$(readlink "$dest")" = "$src" ]; then
    log "Already linked: $dest"
    return
  fi

  if [ -e "$dest" ] || [ -L "$dest" ]; then
    run mkdir -p "$BACKUP_DIR$(dirname "$dest")"
    run mv "$dest" "$BACKUP_DIR$dest"
    action_log "back up: $dest"
  fi

  run ln -s "$src" "$dest"
  action_log "link: $dest"
}

log "Dotfiles root: $ROOT_DIR"
load_settings

if [ "$UPDATE_DOTFILES" -eq 1 ]; then
  update_dotfiles
fi

if [ "$INSTALL_PACKAGES" -eq 1 ]; then
  install_packages
fi

if [ "$CONFIGURE_SYSTEM" -eq 1 ]; then
  configure_system
fi

find "$ROOT_DIR/home" -type f -print | while IFS= read -r src; do
  rel="${src#"$ROOT_DIR/home/"}"
  link_file "$src" "$HOME_DIR/$rel"
done

link_file "$ROOT_DIR/config/dotfiles/settings.conf" "$HOME_DIR/.config/dotfiles/settings.conf"
write_git_user_config

run mkdir -p "$HOME_DIR/.local/bin"
link_file "$ROOT_DIR/scripts/dotctl" "$HOME_DIR/.local/bin/dotctl"
link_file "$ROOT_DIR/scripts/dotfiles-center" "$HOME_DIR/.local/bin/dotfiles-center"
link_file "$ROOT_DIR/scripts/dotfiles-welcome" "$HOME_DIR/.local/bin/dotfiles-welcome"
link_file "$ROOT_DIR/scripts/dotfiles-wallpaper" "$HOME_DIR/.local/bin/dotfiles-wallpaper"

log ""
log "Done. Open a new shell or run: exec \"\$SHELL\""

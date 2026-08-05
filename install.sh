#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
HOME_DIR="${HOME:?HOME is required}"
BACKUP_DIR="$ROOT_DIR/.dotfiles-backup/$(date +%Y%m%d-%H%M%S)"
DRY_RUN=0
INSTALL_PACKAGES=0
INSTALL_AUR=0
UPDATE_DOTFILES=0
CONFIGURE_SYSTEM=0
NOCONFIRM=0
LINK_LIVE_HYPR=0
DOTFILES_NAME="Arch User"
DOTFILES_EMAIL=""

usage() {
  cat <<'EOF'
Usage: ./install.sh [options]

Options:
  --apply         Apply dotfiles without installing or updating packages
  --dry-run       Show actions without changing files
  --packages      Install recommended Arch packages with pacman
  --aur           Install yay, then install packages from packages/aur.txt
  --upd           Update this dotfiles repo, then apply dotfiles
  --system        Configure greetd and enable desktop services
  --all           Update repo, install pacman/AUR packages, configure system, and apply dotfiles
  --noconfirm     Pass --noconfirm to pacman, makepkg, and yay
  --live-hypr     Link Hyprland config even when running inside Hyprland
  -h, --help      Show this help
EOF
}

if [ "$#" -eq 0 ]; then
  usage
  exit 0
fi

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
    --apply)
      :
      ;;
    --dry-run)
      DRY_RUN=1
      ;;
    --packages)
      INSTALL_PACKAGES=1
      ;;
    --aur)
      INSTALL_AUR=1
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
      INSTALL_AUR=1
      CONFIGURE_SYSTEM=1
      ;;
    --noconfirm)
      NOCONFIRM=1
      ;;
    --live-hypr)
      LINK_LIVE_HYPR=1
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

  # Required for SplinterDots wallpaper support.
  # Keep this forced here so wallpapers cannot silently break if the package list is edited.
  if ! printf '%s\n' "${packages[@]}" | grep -qx 'swww'; then
    packages+=("swww")
  fi

  pacman_args=(-S --needed)
  if [ "$NOCONFIRM" -eq 1 ]; then
    pacman_args+=(--noconfirm)
  fi

  if [ "$DRY_RUN" -eq 1 ]; then
    log "[dry-run] sudo pacman ${pacman_args[*]} ${packages[*]}"
    return
  fi

  sudo pacman "${pacman_args[@]}" "${packages[@]}"
}

ensure_rust_toolchain() {
  if ! command -v rustup >/dev/null 2>&1; then
    log "Skipping Rust toolchain setup: rustup was not found."
    return
  fi

  if rustup show active-toolchain >/dev/null 2>&1; then
    log "A Rust toolchain is already configured."
    return
  fi

  if [ "$DRY_RUN" -eq 1 ]; then
    log "[dry-run] rustup default stable"
    return
  fi

  log "Installing the stable Rust toolchain..."
  rustup default stable
}

install_yay() {
  if command -v yay >/dev/null 2>&1; then
    log "yay is already installed."
    return
  fi

  if [ "$IS_ARCH" -ne 1 ]; then
    log "Skipping yay: /etc/arch-release was not found."
    return
  fi

  if [ "$DRY_RUN" -eq 1 ]; then
    if [ "$NOCONFIRM" -eq 1 ]; then
      log "[dry-run] sudo pacman -S --needed --noconfirm base-devel git"
    else
      log "[dry-run] sudo pacman -S --needed base-devel git"
    fi
    log "[dry-run] git clone https://aur.archlinux.org/yay.git /tmp/dotfiles-yay"
    if [ "$NOCONFIRM" -eq 1 ]; then
      log "[dry-run] cd /tmp/dotfiles-yay && makepkg -si --noconfirm"
    else
      log "[dry-run] cd /tmp/dotfiles-yay && makepkg -si"
    fi
    return
  fi

  pacman_args=(-S --needed)
  makepkg_args=(-si)
  if [ "$NOCONFIRM" -eq 1 ]; then
    pacman_args+=(--noconfirm)
    makepkg_args+=(--noconfirm)
  fi

  sudo pacman "${pacman_args[@]}" base-devel git
  rm -rf /tmp/dotfiles-yay
  git clone https://aur.archlinux.org/yay.git /tmp/dotfiles-yay
  (
    cd /tmp/dotfiles-yay
    makepkg "${makepkg_args[@]}"
  )
}

install_aur_packages() {
  if [ "$IS_ARCH" -ne 1 ]; then
    log "Skipping AUR packages: /etc/arch-release was not found."
    return
  fi

  install_yay

  mapfile -t aur_packages < <(sed '/^[[:space:]]*#/d; /^[[:space:]]*$/d' "$ROOT_DIR/packages/aur.txt")

  if [ "${#aur_packages[@]}" -eq 0 ]; then
    log "No AUR packages listed in packages/aur.txt."
    return
  fi

  yay_args=(-S --needed)
  if [ "$NOCONFIRM" -eq 1 ]; then
    yay_args+=(--noconfirm)
  fi

  if [ "$DRY_RUN" -eq 1 ]; then
    log "[dry-run] yay ${yay_args[*]} ${aur_packages[*]}"
    log "[dry-run] sudo chmod a+wr /opt/spotify"
    log "[dry-run] sudo chmod a+wr /opt/spotify/Apps -R"
    return
  fi

  yay "${yay_args[@]}" "${aur_packages[@]}"

  # Spicetify needs write access to Spotify's installation and app files.
  sudo chmod a+wr /opt/spotify
  sudo chmod a+wr /opt/spotify/Apps -R
}

remove_managed_start_hyprland_shadow() {
  local target="/usr/local/bin/start-hyprland"

  if [ "$DRY_RUN" -eq 1 ]; then
    log "[dry-run] remove managed $target if it shadows /usr/bin/start-hyprland"
    return
  fi

  if [ -L "$target" ]; then
    local link_target
    link_target="$(readlink "$target")"
    if [ "$link_target" = "$ROOT_DIR/scripts/start-hyprland" ]; then
      sudo rm -f "$target"
      log "removed managed shadow: $target"
    fi
    return
  fi

  if [ -f "$target" ] && grep -q 'DOTFILES_HYPRLAND_COMMAND' "$target" 2>/dev/null; then
    sudo rm -f "$target"
    log "removed managed shadow: $target"
  fi
}

configure_system() {
  if ! command -v systemctl >/dev/null 2>&1; then
    log "Skipping system setup: systemctl was not found."
    return
  fi

  if [ "$DRY_RUN" -eq 1 ]; then
    log "[dry-run] sudo install -Dm644 $ROOT_DIR/system/greetd/config.toml /etc/greetd/config.toml"
    log "[dry-run] sudo install -Dm644 $ROOT_DIR/system/wayland-sessions/start-hyprland.desktop /usr/share/wayland-sessions/start-hyprland.desktop"
    log "[dry-run] sudo install -Dm755 $ROOT_DIR/scripts/splinter-session /usr/local/bin/splinter-session"
    remove_managed_start_hyprland_shadow
    log "[dry-run] sudo systemctl enable NetworkManager bluetooth greetd"
    return
  fi

  sudo install -Dm644 "$ROOT_DIR/system/greetd/config.toml" /etc/greetd/config.toml
  sudo install -Dm644 "$ROOT_DIR/system/wayland-sessions/start-hyprland.desktop" /usr/share/wayland-sessions/start-hyprland.desktop
  sudo install -Dm755 "$ROOT_DIR/scripts/splinter-session" /usr/local/bin/splinter-session
  remove_managed_start_hyprland_shadow
  sudo systemctl enable NetworkManager bluetooth greetd
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

reenable_welcome_after_update() {
  local disabled_file="$HOME_DIR/.config/dotfiles/welcome-disabled"

  if [ "$UPDATE_DOTFILES" -ne 1 ]; then
    return
  fi

  if [ "$DRY_RUN" -eq 1 ]; then
    log "[dry-run] rm -f $disabled_file"
    return
  fi

  rm -f "$disabled_file"
  log "re-enabled SplinterDots startup popup after update"
}



ensure_wallpaper_directory() {
  local wallpaper_dir="$HOME_DIR/Pictures/Wallpapers"

  if [ "$DRY_RUN" -eq 1 ]; then
    log "[dry-run] mkdir -p $wallpaper_dir"
    return
  fi

  mkdir -p "$wallpaper_dir"
  log "ensured wallpaper directory: $wallpaper_dir"
}

ensure_hyprland_generated_config() {
  local repo_file="$ROOT_DIR/home/.config/hypr/dotfiles-generated.lua"
  local user_file="$HOME_DIR/.config/hypr/dotfiles-generated.lua"

  if [ "$DRY_RUN" -eq 1 ]; then
    log "[dry-run] ensure $repo_file exists"
    log "[dry-run] ensure $user_file exists"
    return
  fi

  mkdir -p "$(dirname "$repo_file")"
  mkdir -p "$(dirname "$user_file")"

  if [ ! -e "$repo_file" ]; then
    cat > "$repo_file" <<'EOF'
-- This file is created automatically by the SplinterDots installer.
-- SplinterDots can write Hyprland settings here.
hl.config({})
EOF
    log "created: $repo_file"
  fi

  if [ ! -e "$user_file" ] && [ ! -L "$user_file" ]; then
    cat > "$user_file" <<'EOF'
-- This file is created automatically by the SplinterDots installer.
-- SplinterDots can write Hyprland settings here.
hl.config({})
EOF
    log "created: $user_file"
  fi
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

remove_legacy_hyprland_links() {
  local rel dest expected
  local legacy_files=(
    "hyprland.conf"
    "colors.conf"
    "dotfiles-generated.conf"
    "keybindings.conf"
    "user.conf"
    "conf/splinter-tools-workspace.conf"
  )

  for rel in "${legacy_files[@]}"; do
    dest="$HOME_DIR/.config/hypr/$rel"
    expected="$ROOT_DIR/home/.config/hypr/$rel"
    if [ -L "$dest" ] && [ "$(readlink "$dest")" = "$expected" ]; then
      run rm "$dest"
      action_log "remove legacy Hyprland link: $dest"
    fi
  done
}

log "Dotfiles root: $ROOT_DIR"
load_settings
ensure_wallpaper_directory
ensure_hyprland_generated_config
remove_legacy_hyprland_links

if [ "$UPDATE_DOTFILES" -eq 1 ]; then
  update_dotfiles
fi

if [ "$INSTALL_PACKAGES" -eq 1 ]; then
  install_packages
  ensure_rust_toolchain
fi

if [ "$INSTALL_AUR" -eq 1 ]; then
  install_aur_packages
fi

if [ "$CONFIGURE_SYSTEM" -eq 1 ]; then
  configure_system
fi

find "$ROOT_DIR/home" -type f -print | while IFS= read -r src; do
  rel="${src#"$ROOT_DIR/home/"}"
  if [ -n "${HYPRLAND_INSTANCE_SIGNATURE:-}" ] && [ "$LINK_LIVE_HYPR" -ne 1 ]; then
    case "$rel" in
      .config/hypr/*)
        log "Skipping live Hyprland config while Hyprland is running: $HOME_DIR/$rel"
        log "Run ./install.sh from a TTY, after reboot, or pass --live-hypr to force it."
        continue
        ;;
    esac
  fi
  link_file "$src" "$HOME_DIR/$rel"
done

link_file "$ROOT_DIR/config/dotfiles/settings.conf" "$HOME_DIR/.config/dotfiles/settings.conf"
write_git_user_config
reenable_welcome_after_update

run mkdir -p "$HOME_DIR/.local/bin"
link_file "$ROOT_DIR/scripts/dotctl" "$HOME_DIR/.local/bin/dotctl"
link_file "$ROOT_DIR/scripts/SplinterDots" "$HOME_DIR/.local/bin/SplinterDots"
link_file "$ROOT_DIR/scripts/splinter-autostart" "$HOME_DIR/.local/bin/splinter-autostart"
link_file "$ROOT_DIR/scripts/splinter-doctor" "$HOME_DIR/.local/bin/splinter-doctor"
link_file "$ROOT_DIR/scripts/splinter-session" "$HOME_DIR/.local/bin/splinter-session"
if [ -L "$HOME_DIR/.local/bin/start-hyprland" ] && [ "$(readlink "$HOME_DIR/.local/bin/start-hyprland")" = "$ROOT_DIR/scripts/start-hyprland" ]; then
  run rm "$HOME_DIR/.local/bin/start-hyprland"
  action_log "remove managed shadow: $HOME_DIR/.local/bin/start-hyprland"
fi
link_file "$ROOT_DIR/scripts/splinter-welcome" "$HOME_DIR/.local/bin/dotfiles-welcome"
link_file "$ROOT_DIR/scripts/splinter-wallpaper" "$HOME_DIR/.local/bin/splinter-wallpaper"
link_file "$ROOT_DIR/scripts/splinter-screenshot" "$HOME_DIR/.local/bin/splinter-screenshot"
link_file "$ROOT_DIR/scripts/splinter-calendar-menu" "$HOME_DIR/.local/bin/splinter-calendar-menu"
link_file "$ROOT_DIR/scripts/splinter-media-menu" "$HOME_DIR/.local/bin/splinter-media-menu"
link_file "$ROOT_DIR/scripts/splinter-cava-read" "$HOME_DIR/.local/bin/splinter-cava-read"
link_file "$ROOT_DIR/scripts/splinter-cava-daemon" "$HOME_DIR/.local/bin/splinter-cava-daemon"
link_file "$ROOT_DIR/scripts/splinter-apply-kitty-theme" "$HOME_DIR/.local/bin/splinter-apply-kitty-theme"
link_file "$ROOT_DIR/scripts/splinter-install-addon" "$HOME_DIR/.local/bin/splinter-install-addon"

log ""
log "Done. Open a new shell or run: exec \"\$SHELL\""

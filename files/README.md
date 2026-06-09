# SplinterDots

Friendly, customizable Hyprland dotfiles for Arch Linux.

This repo is designed around three ideas:

- Change normal settings from **Dotfiles Center** instead of hand-editing config files
- Use switches for on/off settings and dropdowns when there are multiple choices
- Install safely: dry-run first, backups before overwrites
- Keep local machine choices out of Git with `~/.config/dotfiles/local.conf`
- Start from a minimal Arch install and get a complete Hyprland desktop

## Quick Start

Preview what would be installed:

```sh
./install.sh --dry-run
```

Install the dotfiles:

```sh
./install.sh
```

Install recommended Arch packages too:

```sh
./install.sh --packages
```

Install `yay` and AUR packages from `packages/aur.txt`:

```sh
./install.sh --aur
```

Do everything in one run:

```sh
./install.sh --all
```

Run the full setup with automatic package confirmations:

```sh
./install.sh --all --noconfirm
```

On a clean minimal Arch install, `--all` installs Hyprland, Quickshell,
Wofi, Mako, Kitty, Thunar, Zen Browser, PipeWire audio, NetworkManager,
screenshot tools, clipboard tools, wallpaper support, Bluetooth tools,
EasyEffects audio presets, portals, and a graphical login stack.

`--noconfirm` passes automatic confirmation flags to `pacman`, `makepkg`,
and `yay`. It does not bypass sudo password prompts.

Update the dotfiles repo and re-apply links:

```sh
./install.sh --upd
```

After installation, open a new terminal or reload your shell:

```sh
exec "$SHELL"
```

## Dotfiles Center

Open it with:

```sh
dotctl center
```

Dotfiles Center lets you change:

- Hyprland settings with switches, dropdowns, and simple text boxes
- Shortcuts without seeing Hyprland's internal wording
- Wallpaper and accent color
- The Quickshell QML bar
- Default apps such as terminal, browser, file manager, and launcher

The Hyprland tab is generated from:

```text
config/dotfiles/hyprland-options.json
```

To add or rename a setting in the UI, edit that schema file. Boolean
settings become switches. Settings with more than two choices become
dropdowns. Other settings become text boxes.

## Default Shortcuts

All shortcuts use the `Super` / Windows key.

| Shortcut | Action |
| --- | --- |
| `Super + Return` | Open terminal |
| `Super + E` | Open file manager |
| `Super + D` | Open app launcher |
| `Super + B` | Open browser |
| `Super + C` | Close focused window |
| `Super + M` | Exit Hyprland |
| `Super + F` | Toggle fullscreen |
| `Super + V` | Toggle floating window |
| `Super + S` | Select screenshot region, copy it, and save it |
| `Super + Shift + S` | Save full screenshot |
| `Super + W` | Open Dotfiles Center |
| `Super + Shift + R` | Reload Hyprland config |
| `Super + Left Mouse` | Drag windows |
| `Super + Right Mouse` | Resize windows |
| `Super + Arrow keys` | Move focus |
| `Super + Shift + Arrow keys` | Move focused window |
| `Super + 1` through `9` | Switch workspace |
| `Super + Shift + 1` through `9` | Move window to workspace |

## What Gets Linked

The installer links files from `home/` into your home directory.

Important examples:

| Source | Destination |
| --- | --- |
| `home/.zshrc` | `~/.zshrc` |
| `home/.profile` | `~/.profile` |
| `home/.gitconfig` | `~/.gitconfig` |
| `home/.config/hypr/hyprland.conf` | `~/.config/hypr/hyprland.conf` |
| `home/.config/hypr/keybindings.conf` | `~/.config/hypr/keybindings.conf` |
| `home/.config/hypr/colors.conf` | `~/.config/hypr/colors.conf` |
| `home/.config/hypr/dotfiles-generated.conf` | `~/.config/hypr/dotfiles-generated.conf` |
| `home/.config/quickshell/splinterbar/shell.qml` | `~/.config/quickshell/splinterbar/shell.qml` |
| `home/.config/wofi/config` | `~/.config/wofi/config` |
| `home/.config/mako/config` | `~/.config/mako/config` |
| `home/.config/starship.toml` | `~/.config/starship.toml` |
| `config/dotfiles/settings.conf` | `~/.config/dotfiles/settings.conf` |

Existing files are moved into `.dotfiles-backup/<timestamp>/` before links are created.

## Helper Commands

After installing, use:

```sh
dotctl status
dotctl edit
dotctl apply
dotctl update
dotctl all
dotctl packages
dotctl aur
dotctl system
dotctl welcome
dotctl center
dotctl doctor
dotctl uninstall
```

## Quickshell QML Bar

Waybar has been removed from this setup. The bar is now built with
Quickshell/QML and lives here:

```text
home/.config/quickshell/splinterbar/shell.qml
```

Hyprland starts it automatically through:

```text
scripts/dotfiles-hypr-autostart
```

Restart it manually with:

```sh
pkill -x quickshell
quickshell -c splinterbar
```

## Customize

Edit the main settings file:

```sh
$EDITOR config/dotfiles/settings.conf
```

After changing settings, re-run:

```sh
dotctl apply
```

For machine-specific settings that should not be committed, create:

```sh
mkdir -p ~/.config/dotfiles
$EDITOR ~/.config/dotfiles/local.conf
```

Example `local.conf`:

```sh
DOTFILES_THEME="dark"
DOTFILES_EDITOR="nano"
DOTFILES_BROWSER="zen-browser"
DOTFILES_ENABLE_STARSHIP="1"
```

`DOTFILES_NAME` and `DOTFILES_EMAIL` are used to generate
`~/.config/dotfiles/git-user.inc`, which is included by `~/.gitconfig`.

## Audio And Bluetooth

`--all` installs and enables Bluetooth with `bluez`, `bluez-utils`, `blueman`,
and the `bluetooth` service. Open Blueman from the launcher to pair devices.

EasyEffects starts with Hyprland and includes two presets:

| Preset | Location | Purpose |
| --- | --- | --- |
| `Dotfiles Output` | Output tab | Gentle EQ and limiter for clearer desktop audio |
| `Dotfiles Mic` | Input tab | Noise gate, voice compression, voice EQ, and limiter |

Open EasyEffects from the launcher, go to the Output and Input tabs, and select
the matching preset.

## Hyprland Crash Recovery

If Hyprland crashes or prints many Aquamarine errors on a clean install, switch
to a TTY with `Ctrl + Alt + F3`, log in, and run:

```sh
dotctl doctor
```

The graphical login starts Hyprland through the packaged
`/usr/bin/start-hyprland`. If Hyprland still shows a startup warning, run:

```sh
dotctl system
```

For virtual machines or emergency fallback systems, you can allow software
rendering by adding this to `~/.config/dotfiles/local.conf`:

```sh
DOTFILES_ALLOW_SOFTWARE_RENDERER="1"
```

The installer avoids replacing `~/.config/hypr/*` while Hyprland is already
running because live config reloads can crash a fragile first session. Run the
installer from a TTY or after reboot for safest Hyprland config updates. Use
`--live-hypr` only when you intentionally want to force it.

## Layout

```text
.
├── config/dotfiles/settings.conf          # Main user-facing settings
├── config/dotfiles/hyprland-options.json  # Hyprland settings shown in Dotfiles Center
├── home/                                  # Files linked into $HOME
├── packages/arch.txt                      # Recommended pacman packages
├── packages/aur.txt                       # Optional AUR packages installed with yay
├── scripts/dotctl                         # Helper command
└── install.sh                             # Safe installer
```

## Uninstall

This removes only symlinks that point back to this repo:

```sh
dotctl uninstall
```

Backups are kept in `.dotfiles-backup/` so you can restore manually if needed.

# Arch Dotfiles

Friendly, customizable Hyprland dotfiles for Arch Linux.

This repo is designed around three ideas:

- Change settings in one obvious place: `config/dotfiles/settings.conf`
- Install safely: dry-run first, backups before overwrites
- Keep local machine choices out of Git: use `~/.config/dotfiles/local.conf`
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

On a clean minimal Arch install, `--all` installs Hyprland, Waybar, Wofi,
Mako, Kitty, Thunar, Firefox, PipeWire audio, NetworkManager, screenshot tools,
clipboard tools, wallpaper support, Bluetooth tools, EasyEffects audio presets,
portals, and a graphical login stack. It
also installs the greetd config from `system/greetd/config.toml` and enables
NetworkManager, Bluetooth, and greetd. It also installs `yay` if missing, then
installs any AUR packages listed in `packages/aur.txt`.

`--noconfirm` passes automatic confirmation flags to `pacman`, `makepkg`, and
`yay`. It does not bypass sudo password prompts.

Update the dotfiles repo and re-apply links:

```sh
./install.sh --upd
```

After installation, open a new terminal or reload your shell:

```sh
exec "$SHELL"
```

On graphical desktop login, Dotfiles Center appears. It explains keybinds and
lets you change wallpaper, colors, apps, and common Hyprland keybinds without
hand-editing config files. Click `Don't show this on startup` to disable it.

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
DOTFILES_BROWSER="firefox"
DOTFILES_ENABLE_STARSHIP="1"
```

`DOTFILES_NAME` and `DOTFILES_EMAIL` are used to generate
`~/.config/dotfiles/git-user.inc`, which is included by `~/.gitconfig`.

## What Gets Linked

The installer links files from `home/` into your home directory:

| Source | Destination |
| --- | --- |
| `home/.zshrc` | `~/.zshrc` |
| `home/.profile` | `~/.profile` |
| `home/.gitconfig` | `~/.gitconfig` |
| `home/.config/hypr/hyprland.conf` | `~/.config/hypr/hyprland.conf` |
| `home/.config/hypr/keybindings.conf` | `~/.config/hypr/keybindings.conf` |
| `home/.config/hypr/colors.conf` | `~/.config/hypr/colors.conf` |
| `home/.config/waybar/config` | `~/.config/waybar/config` |
| `home/.config/waybar/style.css` | `~/.config/waybar/style.css` |
| `home/.config/wofi/config` | `~/.config/wofi/config` |
| `home/.config/wofi/style.css` | `~/.config/wofi/style.css` |
| `home/.config/mako/config` | `~/.config/mako/config` |
| `home/.config/starship.toml` | `~/.config/starship.toml` |
| `home/.config/autostart/dotfiles-welcome.desktop` | `~/.config/autostart/dotfiles-welcome.desktop` |
| `home/.local/share/easyeffects/output/Dotfiles Output.json` | `~/.local/share/easyeffects/output/Dotfiles Output.json` |
| `home/.local/share/easyeffects/input/Dotfiles Mic.json` | `~/.local/share/easyeffects/input/Dotfiles Mic.json` |
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
```

## Default Hyprland Keybinds

All keybinds use the `Super` / Windows key.

| Keybind | Action |
| --- | --- |
| `Super + Return` | Open terminal |
| `Super + E` | Open file manager |
| `Super + D` | Open app launcher |
| `Super + B` | Open browser |
| `Super + C` | Close focused window |
| `Super + M` | Exit Hyprland |
| `Super + F` | Toggle fullscreen |
| `Super + V` | Toggle floating window |
| `Super + S` | Select screenshot region and copy it |
| `Super + Shift + S` | Save full screenshot |
| `Super + W` | Open Dotfiles Center |
| `Super + Shift + R` | Reload Hyprland config |
| `Super + Arrow keys` | Move focus |
| `Super + Shift + Arrow keys` | Move focused window |
| `Super + 1` through `9` | Switch workspace |
| `Super + Shift + 1` through `9` | Move window to workspace |

Use `dotctl center` to change wallpaper, colors, apps, and keybinds
graphically.

## Audio And Bluetooth

`--all` installs and enables Bluetooth with `bluez`, `bluez-utils`, `blueman`,
and the `bluetooth` service. Open Blueman from the launcher to pair devices.

EasyEffects starts with Hyprland and includes two presets:

| Preset | Location | Purpose |
| --- | --- | --- |
| `Dotfiles Output` | Output tab | Gentle EQ and limiter for clearer desktop audio |
| `Dotfiles Mic` | Input tab | Noise gate, voice compression, voice EQ, and limiter |

Open EasyEffects from the launcher, go to the Output and Input tabs, and select
the matching preset. Presets are deliberately conservative because every mic,
headset, and speaker is different.

## Hyprland Crash Recovery

If Hyprland crashes or prints many Aquamarine errors on a clean install, switch
to a TTY with `Ctrl + Alt + F3`, log in, and run:

```sh
dotctl doctor
```

Aquamarine errors are usually graphics/session backend problems. This setup
installs the common Mesa/Vulkan userspace packages for Intel and AMD graphics,
but NVIDIA systems still need the matching `nvidia` or `nvidia-dkms` driver
stack for the installed kernel.

The installer avoids replacing `~/.config/hypr/*` while Hyprland is already
running because live config reloads can crash a fragile first session. Run the
installer from a TTY or after reboot for safest Hyprland config updates. Use
`--live-hypr` only when you intentionally want to force a live Hyprland config
replacement.

## Layout

```text
.
├── config/dotfiles/settings.conf  # Main user-facing settings
├── home/                          # Files linked into $HOME
├── packages/arch.txt              # Recommended pacman packages
├── packages/aur.txt               # Optional AUR packages installed with yay
├── scripts/dotctl                 # Helper command
└── install.sh                     # Safe installer
```

## Uninstall

This removes only symlinks that point back to this repo:

```sh
dotctl uninstall
```

Backups are kept in `.dotfiles-backup/` so you can restore manually if needed.

To re-enable the welcome tour after clicking `Don't show again`, remove:

```sh
rm ~/.config/dotfiles/welcome-disabled
```

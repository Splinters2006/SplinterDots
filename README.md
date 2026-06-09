# Arch Dotfiles

Friendly, customizable dotfiles for Arch Linux.

This repo is designed around three ideas:

- Change settings in one obvious place: `config/dotfiles/settings.conf`
- Install safely: dry-run first, backups before overwrites
- Keep local machine choices out of Git: use `~/.config/dotfiles/local.conf`

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

After installation, open a new terminal or reload your shell:

```sh
exec "$SHELL"
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
DOTFILES_EDITOR="nvim"
DOTFILES_BROWSER="firefox"
DOTFILES_ENABLE_STARSHIP="1"
```

`DOTFILES_NAME` and `DOTFILES_EMAIL` are used to generate
`~/.config/dotfiles/git-user.inc`, which is included by `~/.gitconfig`.

## What Gets Linked

The installer links files from `home/` into your home directory:

| Source | Destination |
| --- | --- |
| `home/.bashrc` | `~/.bashrc` |
| `home/.profile` | `~/.profile` |
| `home/.gitconfig` | `~/.gitconfig` |
| `home/.config/starship.toml` | `~/.config/starship.toml` |
| `home/.config/nvim/init.lua` | `~/.config/nvim/init.lua` |
| `config/dotfiles/settings.conf` | `~/.config/dotfiles/settings.conf` |

Existing files are moved into `.dotfiles-backup/<timestamp>/` before links are created.

## Helper Commands

After installing, use:

```sh
dotctl status
dotctl edit
dotctl apply
dotctl packages
```

## Layout

```text
.
├── config/dotfiles/settings.conf  # Main user-facing settings
├── home/                          # Files linked into $HOME
├── packages/arch.txt              # Recommended pacman packages
├── scripts/dotctl                 # Helper command
└── install.sh                     # Safe installer
```

## Uninstall

This removes only symlinks that point back to this repo:

```sh
dotctl uninstall
```

Backups are kept in `.dotfiles-backup/` so you can restore manually if needed.

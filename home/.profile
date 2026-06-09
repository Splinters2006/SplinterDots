# ~/.profile

DOTFILES_CONFIG_HOME="${XDG_CONFIG_HOME:-$HOME/.config}/dotfiles"
DOTFILES_REPO_CONFIG="$HOME/.config/dotfiles/settings.conf"

if [ -f "$DOTFILES_REPO_CONFIG" ]; then
  # shellcheck disable=SC1090
  . "$DOTFILES_REPO_CONFIG"
fi

if [ -f "$DOTFILES_CONFIG_HOME/local.conf" ]; then
  # shellcheck disable=SC1091
  . "$DOTFILES_CONFIG_HOME/local.conf"
fi

export EDITOR="${DOTFILES_EDITOR:-nvim}"
export VISUAL="$EDITOR"
export BROWSER="${DOTFILES_BROWSER:-firefox}"

if [ -n "${DOTFILES_EXTRA_PATH:-}" ]; then
  export PATH="$DOTFILES_EXTRA_PATH:$PATH"
fi

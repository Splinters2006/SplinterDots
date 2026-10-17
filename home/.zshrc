# SplinterDots Zsh entry point. Also works when the full config is loaded via ZDOTDIR.
if [[ -r "${XDG_CONFIG_HOME:-$HOME/.config}/zsh/.zshrc" ]]; then
  source "${XDG_CONFIG_HOME:-$HOME/.config}/zsh/.zshrc"
fi
# ~/.bashrc

if [ -f "$HOME/.profile" ]; then
  # shellcheck disable=SC1090
  . "$HOME/.profile"
fi

alias grep='grep --color=auto'
alias mkdir='mkdir -p'
alias ..='cd ..'
alias ...='cd ../..'

if command -v eza >/dev/null 2>&1; then
  alias ls="${DOTFILES_ALIAS_LS:-eza --icons --group-directories-first}"
  alias ll='ls -lah'
  alias la='ls -A'
else
  alias ls='ls --color=auto'
  alias ll='ls -lah'
  alias la='ls -A'
fi

if command -v bat >/dev/null 2>&1; then
  alias cat="${DOTFILES_ALIAS_CAT:-bat}"
fi

if [ "${DOTFILES_ENABLE_STARSHIP:-1}" = "1" ] && command -v starship >/dev/null 2>&1; then
  eval "$(starship init bash)"
fi

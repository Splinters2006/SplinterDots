# ~/.zshrc

if [ -f "$HOME/.profile" ]; then
  # shellcheck disable=SC1090
  . "$HOME/.profile"
fi

autoload -Uz compinit
compinit

setopt auto_cd
setopt auto_pushd
setopt correct
setopt hist_ignore_all_dups
setopt hist_reduce_blanks
setopt share_history

HISTFILE="${XDG_STATE_HOME:-$HOME/.local/state}/zsh/history"
HISTSIZE=10000
SAVEHIST=10000
mkdir -p "$(dirname "$HISTFILE")"

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

if command -v zoxide >/dev/null 2>&1; then
  eval "$(zoxide init zsh)"
fi

if [ "${DOTFILES_ENABLE_STARSHIP:-1}" = "1" ] && command -v starship >/dev/null 2>&1; then
  eval "$(starship init zsh)"
fi
alias spdbuild='cd ~/SplinterDots && cargo build --release --manifest-path Cargo.toml -p SplinterDots && (pkill -f SplinterDots || true) && ./target/release/SplinterDots'
alias spdbuild='cd ~/SplinterDots && cargo build --release --manifest-path Cargo.toml -p SplinterDots && (pkill -f SplinterDots || true) && ./target/release/SplinterDots'

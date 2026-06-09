# Cargo Dotfiles Center

This patch makes Dotfiles Center a normal Cargo workspace project inside the dotfiles repo.

It adds:

- `Cargo.toml` at the repo root
- `tools/dotfiles-center/Cargo.toml`
- `tools/dotfiles-center/src/main.rs`
- a small Bash launcher at `scripts/dotfiles-center`

It updates:

- `packages/arch.txt`: removes `tk`, adds `rust` and `cargo`
- `.gitignore`: ignores `/target/`

Apply from the repo root:

```bash
./apply_cargo_dotfiles_center.sh
```

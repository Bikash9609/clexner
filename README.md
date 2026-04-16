# cachectl

`cachectl` is a Rust CLI/TUI tool for scanning and cleaning common local dev caches.

## Requirements

- Rust toolchain (Cargo)
- macOS (current cache paths in this project are macOS-style)

## Run

```bash
cargo run
```

This starts the interactive TUI flow (detect -> scan -> cleanup UI).

## Useful Commands

```bash
# List detected tools
cargo run -- list-tools

# List cache targets with sizes
cargo run -- list-caches

# Print full scan report
cargo run -- scan

# Clean selected targets by id (asks for confirmation)
cargo run -- clean --targets npm_cache,yarn_cache

# Clean selected targets without prompt
cargo run -- clean --targets npm_cache,yarn_cache --confirm

# Include recursive .venv discovery from home dir
cargo run -- --include-venv
```

## Cleaning Supported Right Now

Targets are auto-included based on detected installed tools.

- `npm_cache` -> `~/.npm`
- `yarn_cache` -> `~/Library/Caches/Yarn`
- `pnpm_store` -> `~/Library/pnpm/store`
- `uv_cache` -> `~/.cache/uv`
- `pip_cache` -> `~/Library/Caches/pip`
- `poetry_cache` -> `~/Library/Caches/pypoetry`
- `pipx_cache` -> `~/.cache/pipx`
- `cargo_registry` -> `~/.cargo/registry`
- `cargo_git` -> `~/.cargo/git`
- `rustup_downloads` -> `~/.rustup/downloads`
- `rustup_tmp` -> `~/.rustup/tmp`
- `docker_cache` -> `~/Library/Caches/com.docker.docker`
- `wasp_cache` -> `~/.wasp/cache`
- `venv_dirs` (optional) -> recursive `.venv` directories from `~` (enabled via `--include-venv`)

## What It Does Not Support Yet

- Linux/Windows cache path mapping
- Custom cache path config via file/env
- Per-project lock/safety rules before delete
- Dry-run mode for the `clean` command
- Automated backup/restore

## Public Repo Privacy Check

Quick scan in this repo did not find obvious personal details (email, username strings, hardcoded user paths, secrets).

Good to keep:
- `target/` out of version control (already in `.gitignore`)
- local env/secret files out of repo

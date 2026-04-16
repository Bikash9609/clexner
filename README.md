# cachectl

<img width="1536" height="1024" alt="image" src="https://github.com/user-attachments/assets/01ed7a83-3f4c-495d-a4ca-d7cbc58ef6d0" />


`cachectl` is a Rust CLI/TUI tool for scanning and cleaning common local dev caches.

## Requirements

- Rust toolchain (Cargo)
- macOS (current cache paths in this project are macOS-style)

## Run

```bash
cargo run
```

This starts the interactive TUI flow (detect -> scan -> cleanup UI).

Or run the explicit TUI subcommand:

```bash
cargo run -- tui
```

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
cargo run -- --include-venv scan

# Include recursive .venv discovery and open TUI
cargo run -- --include-venv tui
```

## Cleaning Supported Right Now

Tools supported for cleanup right now (with their cache targets/paths):

- `npm` -> `npm_cache` -> `~/.npm`
- `yarn` -> `yarn_cache` -> `~/Library/Caches/Yarn`
- `pnpm` -> `pnpm_store` -> `~/Library/pnpm/store`
- `uv` -> `uv_cache` -> `~/.cache/uv`
- `pip` -> `pip_cache` -> `~/Library/Caches/pip`
- `poetry` -> `poetry_cache` -> `~/Library/Caches/pypoetry`
- `pipx` -> `pipx_cache` -> `~/.cache/pipx`
- `cargo` -> `cargo_registry` + `cargo_git` -> `~/.cargo/registry`, `~/.cargo/git`
- `rustup` -> `rustup_downloads` + `rustup_tmp` -> `~/.rustup/downloads`, `~/.rustup/tmp`
- `docker` -> `docker_cache` -> `~/Library/Caches/com.docker.docker`
- `wasp` -> `wasp_cache` -> `~/.wasp/cache`
- `.venv` (optional) -> `venv_dirs` -> recursive `.venv` directories from `~` (enabled via `--include-venv`)

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

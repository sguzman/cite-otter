# Post-Change Checklist (Manual)

Run these commands after a change is completed and confirmed to compile. Run
from the repo root.

Prereqs: `just`, `taplo`, and `biome` must be installed.

## Script

- `./scripts/post-change.sh fast`
- `./scripts/post-change.sh full`
- `./scripts/post-change.sh full-benchmarks`

## Fast Verification (default)

- `just verify-fast`

## Full Verification

- `just verify-full`

## Format Rust

- `cargo fmt`

## Format TOML

- `taplo fmt`

## Validate TOML

- `taplo validate`

## Format JSON (Biome)

- `biome format --write <files-or-directories>`
  - Example: `biome format --write path/to/file.json`
  - Example: `biome format --write .`

## Format Markdown (rumdl)

- `rumdl fmt .`

## Link Check (Lychee)

- `lychee --config lychee.toml .`

## Spelling (typos)

- `typos --config typos.toml`

## Tests

- `cargo test`

## Benchmarks (hyperfine, Optional and Long-Running)

- `just bench-ruby-parity`
- `just bench-ruby-parity-full`
- `just bench-rust-baseline`

## Docs

- Update any docs that changed behavior/config/API

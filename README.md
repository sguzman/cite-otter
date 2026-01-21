# Cite-Otter

Cite-Otter is a Rust-based reimagining of the Ruby `AnyStyle` reference parser. It lives under `tmp/anystyle` as a reference implementation (see `tmp/anystyle/README.md`) and will grow into a Rust-native tool that ingests unstructured citation text and returns structured metadata.

## What’s the goal

- Match the core features of `AnyStyle` in Rust: reference parsing, finder/train workflows, and JSON output.
- Use idiomatic Rust for parsing (e.g., leveraging `nom`, `regex`, or ML bindings later).
- Surface a CLI and, later, library entry points so downstream tools can consume parsed citations.

## Current status

- Repository scaffolded with `Cargo.toml`/`src/`.
- No code exists yet; the focus before development is documenting requirements, design, and inspiration.
- The Ruby `AnyStyle` source in `tmp/anystyle` serves as a design and behavioral reference.

## Getting started (for future work)

1. Install Rust 2024 edition (`rustup show` should align with `rust-toolchain.toml`).
2. Build the workspace via `cargo build`.
3. Review `tmp/anystyle` for example CLI commands and expected output shapes.

## How we’ll structure the implementation

- `src/cli.rs` (future): CLI argument parsing, commands for `parse`, `find`, `train`, etc.
- `src/parser/` (future): Tokenization and label normalization logic, taking cues from `AnyStyle`’s training sets.
- `src/format/` (future): Output adapters (JSON, YAML) modeled after `AnyStyle`’s CLI flags.
- `docs/`: expansion planned once feature requirements solidify.

## References

- `tmp/anystyle/README.md` — official Ruby project documentation.
- `tmp/anystyle/res/` — training data sets that inform parsing expectations.

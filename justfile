set shell := ["bash", "-uc"]

default:
	@just --list

# Context
context:
	files-to-prompt --ignore target/ --ignore .git/ --markdown --line-numbers --extension yaml --extension yml --extension rs --extension toml --extension md . > ~/Downloads/all.txt

context-by-file:
	files-to-prompt --ignore target/ --ignore .git/ --markdown --line-numbers --extension rs . > ~/Downloads/rs.txt
	files-to-prompt --ignore target/ --ignore .git/ --markdown --line-numbers --extension md . > ~/Downloads/md.txt
	files-to-prompt --ignore target/ --ignore .git/ --markdown --line-numbers --extension toml . > ~/Downloads/toml.txt
	files-to-prompt --ignore target/ --ignore .git/ --markdown --line-numbers --extension yaml --extension yml . > ~/Downloads/yaml.txt

major:
	cargo release major --verbose --workspace --execute --no-publish --no-push --no-confirm
	git push
	git push --tags

minor:
	cargo release minor --verbose --workspace --execute --no-publish --no-push --no-confirm
	git push
	git push --tags

patch:
	cargo release patch --verbose --workspace --execute --no-publish --no-push --no-confirm
	git push
	git push --tags

# Copy over template files
template dir:
	cp -riv {{dir}}/docs .
	cp -iv {{dir}}/*.toml .
	cp -iv {{dir}}/about.hbs .
	cp -iv {{dir}}/biome.json .
	cp -iv {{dir}}/.gitignore .
	cp -iv {{dir}}/LICENSE .

# Build
build:
	cargo build

# Format
fmt:
	cargo fmt
	taplo fmt
	biome check --write .

# Validate
typos:
	typos --config typos.toml
links:
	lychee --config lychee.toml .
validate:
	taplo validate

# Test
test:
	cargo test

# Lint
clippy:
	cargo clippy -- -D warnings

# CI-style format check (no writes)
fmt-check:
	cargo fmt --check
	taplo fmt --check
	biome check .

# Docs (optional but useful)
doc:
	cargo doc --no-deps

# Security (Rust)
audit:
	cargo audit

deny:
	cargo deny check

# Coverage (Rust)
# (Requires cargo-llvm-cov installed)
coverage:
	cargo llvm-cov --workspace --all-features --lcov --output-path target/coverage.lcov

coverage-html:
	cargo llvm-cov --workspace --all-features --open

# "CI local" = what you'd run before pushing
ci: fmt-check typos links validate clippy test doc build

# Your existing "all" is fine; consider swapping to ci if you want strict checks by default
all: fmt typos links validate test build


#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BOOK_PATH="${BOOK_PATH:-${ROOT_DIR}/tmp/book.txt}"
RUBY_REPO="${RUBY_REPO:-${ROOT_DIR}/tmp/anystyle}"
OUT_DIR="${OUT_DIR:-${ROOT_DIR}/target/reports}"
HYPERFINE_BIN="${HYPERFINE_BIN:-hyperfine}"
RUST_CMD="${RUST_CMD:-cargo run --quiet --bin cite-otter --}"

if ! command -v "${HYPERFINE_BIN}" >/dev/null 2>&1; then
  echo "error: hyperfine not found (install hyperfine and retry)" >&2
  exit 1
fi

if [[ ! -f "${BOOK_PATH}" ]]; then
  echo "error: benchmark input not found at ${BOOK_PATH}" >&2
  exit 1
fi

if [[ ! -d "${RUBY_REPO}" ]]; then
  echo "error: Ruby AnyStyle repo not found at ${RUBY_REPO}" >&2
  exit 1
fi

if command -v anystyle >/dev/null 2>&1; then
  ANYSTYLE_CMD="anystyle"
else
  echo "error: anystyle-cli not available (install anystyle-cli or add it to PATH)" >&2
  exit 1
fi

mkdir -p "${OUT_DIR}"

HYPERFINE_EXPORT="${OUT_DIR}/benchmark-ruby-parity.json"

${HYPERFINE_BIN} \
  --warmup 1 \
  --min-runs 3 \
  --export-json "${HYPERFINE_EXPORT}" \
  --command-name "ruby:parse-json" "${ANYSTYLE_CMD} -f json parse \"${BOOK_PATH}\"" \
  --command-name "rust:parse-json" "${RUST_CMD} parse --format json \"${BOOK_PATH}\"" \
  --command-name "ruby:parse-bibtex" "${ANYSTYLE_CMD} -f bibtex parse \"${BOOK_PATH}\"" \
  --command-name "rust:parse-bibtex" "${RUST_CMD} parse --format bibtex \"${BOOK_PATH}\"" \
  --command-name "ruby:parse-csl" "${ANYSTYLE_CMD} -f csl parse \"${BOOK_PATH}\"" \
  --command-name "rust:parse-csl" "${RUST_CMD} parse --format csl \"${BOOK_PATH}\"" \
  --command-name "ruby:find-json" "${ANYSTYLE_CMD} -f json find \"${BOOK_PATH}\"" \
  --command-name "rust:find-json" "${RUST_CMD} find --format json \"${BOOK_PATH}\""

echo "hyperfine report written to ${HYPERFINE_EXPORT}"

#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BOOK_PATH="${BOOK_PATH:-${ROOT_DIR}/tmp/book.txt}"
RUBY_REPO="${RUBY_REPO:-${ROOT_DIR}/tmp/anystyle}"
OUT_DIR="${OUT_DIR:-${ROOT_DIR}/target/reports}"
HYPERFINE_BIN="${HYPERFINE_BIN:-hyperfine}"
HYPERFINE_ARGS="${HYPERFINE_ARGS:-}"
RUST_CMD="${RUST_CMD:-cargo run --quiet --bin cite-otter --}"
PARSER_PATTERN="${PARSER_PATTERN:-${ROOT_DIR}/tmp/anystyle/res/parser/core.xml}"
FINDER_PATTERN="${FINDER_PATTERN:-${ROOT_DIR}/tmp/anystyle/res/finder/*.ttx}"
FAST_RUNS="${FAST_RUNS:-3}"
TRAINING_RUNS="${TRAINING_RUNS:-1}"
ENABLE_TRAINING_BENCHMARKS="${ENABLE_TRAINING_BENCHMARKS:-1}"

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

if [[ -n "${ANYSTYLE_CMD:-}" ]]; then
  :
elif command -v anystyle >/dev/null 2>&1; then
  ANYSTYLE_CMD="anystyle"
elif command -v anystyle-cli >/dev/null 2>&1; then
  ANYSTYLE_CMD="anystyle-cli"
else
  echo "error: anystyle-cli not available (install anystyle-cli or add it to PATH)" >&2
  exit 1
fi

mkdir -p "${OUT_DIR}"

HYPERFINE_EXPORT="${OUT_DIR}/benchmark-ruby-parity.json"
HYPERFINE_TRAIN_EXPORT="${OUT_DIR}/benchmark-ruby-parity-training.json"

${HYPERFINE_BIN} ${HYPERFINE_ARGS} \
  --warmup 1 \
  --min-runs "${FAST_RUNS}" \
  --export-json "${HYPERFINE_EXPORT}" \
  --command-name "ruby:parse-json" "${ANYSTYLE_CMD} -f json parse \"${BOOK_PATH}\"" \
  --command-name "rust:parse-json" "${RUST_CMD} parse -o json \"${BOOK_PATH}\"" \
  --command-name "ruby:parse-bibtex" "${ANYSTYLE_CMD} -f bibtex parse \"${BOOK_PATH}\"" \
  --command-name "rust:parse-bibtex" "${RUST_CMD} parse -o bibtex \"${BOOK_PATH}\"" \
  --command-name "ruby:parse-csl" "${ANYSTYLE_CMD} -f csl parse \"${BOOK_PATH}\"" \
  --command-name "rust:parse-csl" "${RUST_CMD} parse -o csl \"${BOOK_PATH}\"" \
  --command-name "ruby:find-json" "${ANYSTYLE_CMD} -f json find \"${BOOK_PATH}\"" \
  --command-name "rust:find-json" "${RUST_CMD} find -o json \"${BOOK_PATH}\""

if [[ "${ENABLE_TRAINING_BENCHMARKS}" == "1" ]]; then
  RUBY_TRAIN_CMD="${RUBY_TRAIN_CMD:-${ANYSTYLE_CMD} train \"${PARSER_PATTERN}\" \"${FINDER_PATTERN}\"}"
  RUBY_CHECK_CMD="${RUBY_CHECK_CMD:-${ANYSTYLE_CMD} check \"${PARSER_PATTERN}\" \"${FINDER_PATTERN}\"}"
  RUBY_DELTA_CMD="${RUBY_DELTA_CMD:-${ANYSTYLE_CMD} delta \"${PARSER_PATTERN}\" \"${FINDER_PATTERN}\"}"
  ${HYPERFINE_BIN} ${HYPERFINE_ARGS} \
    --warmup 0 \
    --min-runs "${TRAINING_RUNS}" \
    --export-json "${HYPERFINE_TRAIN_EXPORT}" \
    --command-name "ruby:train" "${RUBY_TRAIN_CMD}" \
    --command-name "rust:train" "${RUST_CMD} train --parser-dataset \"${PARSER_PATTERN}\" --finder-dataset \"${FINDER_PATTERN}\"" \
    --command-name "ruby:check" "${RUBY_CHECK_CMD}" \
    --command-name "rust:check" "${RUST_CMD} check --parser-dataset \"${PARSER_PATTERN}\" --finder-dataset \"${FINDER_PATTERN}\"" \
    --command-name "ruby:delta" "${RUBY_DELTA_CMD}" \
    --command-name "rust:delta" "${RUST_CMD} delta --parser-dataset \"${PARSER_PATTERN}\" --finder-dataset \"${FINDER_PATTERN}\""
  echo "hyperfine training report written to ${HYPERFINE_TRAIN_EXPORT}"
fi

echo "hyperfine report written to ${HYPERFINE_EXPORT}"

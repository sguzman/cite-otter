#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BOOK_PATH="${BOOK_PATH:-${ROOT_DIR}/tmp/book.txt}"
OUT_DIR="${OUT_DIR:-${ROOT_DIR}/target/reports}"
HYPERFINE_BIN="${HYPERFINE_BIN:-hyperfine}"
HYPERFINE_ARGS="${HYPERFINE_ARGS:-}"
RUST_CMD="${RUST_CMD:-cargo run --quiet --bin cite-otter --}"
HYPERFINE_SUMMARIZER="${HYPERFINE_SUMMARIZER:-cargo run --quiet --bin summarize_hyperfine --}"
FAST_RUNS="${FAST_RUNS:-3}"

if ! command -v "${HYPERFINE_BIN}" >/dev/null 2>&1; then
  echo "error: hyperfine not found (install hyperfine and retry)" >&2
  exit 1
fi

if [[ ! -f "${BOOK_PATH}" ]]; then
  echo "error: benchmark input not found at ${BOOK_PATH}" >&2
  exit 1
fi

mkdir -p "${OUT_DIR}"

HYPERFINE_EXPORT="${OUT_DIR}/benchmark-rust-baseline.json"
HYPERFINE_SUMMARY="${OUT_DIR}/benchmark-rust-baseline-summary.md"

run_summarizer() {
  local input_path="$1"
  local output_path="$2"
  eval "${HYPERFINE_SUMMARIZER} \"${input_path}\" \"${output_path}\""
}

${HYPERFINE_BIN} ${HYPERFINE_ARGS} \
  --warmup 1 \
  --min-runs "${FAST_RUNS}" \
  --export-json "${HYPERFINE_EXPORT}" \
  --command-name "rust:parse-json" "${RUST_CMD} parse -o json \"${BOOK_PATH}\"" \
  --command-name "rust:parse-bibtex" "${RUST_CMD} parse -o bibtex \"${BOOK_PATH}\"" \
  --command-name "rust:parse-csl" "${RUST_CMD} parse -o csl \"${BOOK_PATH}\"" \
  --command-name "rust:find-json" "${RUST_CMD} find -o json \"${BOOK_PATH}\"" \
  --command-name "rust:sample-json" "${RUST_CMD} sample -f json" \
  --command-name "rust:sample-bibtex" "${RUST_CMD} sample -f bibtex" \
  --command-name "rust:sample-csl" "${RUST_CMD} sample -f csl"

run_summarizer "${HYPERFINE_EXPORT}" "${HYPERFINE_SUMMARY}"

echo "hyperfine report written to ${HYPERFINE_EXPORT}"
echo "hyperfine summary written to ${HYPERFINE_SUMMARY}"

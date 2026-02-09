#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MODE="${1:-fast}"

usage() {
  cat <<'EOF'
usage: scripts/post-change.sh [fast|full|full-benchmarks]

Modes:
  fast             Default local checks (no long parity/benchmark runs)
  full             Full CI checks + Ruby format parity comparison
  full-benchmarks  Full checks plus long hyperfine benchmarks
EOF
}

if ! command -v just >/dev/null 2>&1; then
  echo "error: just is required to run post-change checks" >&2
  exit 1
fi

cd "${ROOT_DIR}"

case "${MODE}" in
  fast)
    just verify-fast
    ;;
  full)
    just verify-full
    ;;
  full-benchmarks)
    just verify-full-with-benchmarks
    ;;
  *)
    usage >&2
    exit 1
    ;;
esac

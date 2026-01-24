#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REPORT_DIR="$ROOT/target/reports"
RUBY_DIR="$REPORT_DIR/ruby-format"
REPORT_PATH="$REPORT_DIR/ruby-format-diff.txt"

mkdir -p "$REPORT_DIR"

ruby "$ROOT/scripts/generate_ruby_format_fixtures.rb"

{
  echo "ruby format diff report"
  echo "generated: $(date -u +"%Y-%m-%dT%H:%M:%SZ")"
  echo ""
  for name in core-csl core-bibtex csl bibtex; do
    left="$ROOT/tests/fixtures/format/${name}.txt"
    right="$RUBY_DIR/${name}.txt"
    echo "== diff: $name =="
    if [ -f "$left" ] && [ -f "$right" ]; then
      diff -u "$left" "$right" || true
    else
      echo "missing fixture(s) for $name"
    fi
    echo ""
  done
} > "$REPORT_PATH"

echo "wrote $REPORT_PATH"

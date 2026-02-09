#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REPORT_DIR="$ROOT/target/reports"
RUBY_DIR="$REPORT_DIR/ruby-format"
REPORT_PATH="$REPORT_DIR/ruby-format-diff.txt"
SUMMARY_PATH="$REPORT_DIR/ruby-format-summary.json"
GENERATED_AT="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

mkdir -p "$REPORT_DIR"

ruby "$ROOT/scripts/generate_ruby_format_fixtures.rb"

{
  echo "ruby format diff report"
  echo "generated: ${GENERATED_AT}"
  echo ""
} > "$REPORT_PATH"

json_escape() {
  local value="$1"
  value="${value//\\/\\\\}"
  value="${value//\"/\\\"}"
  value="${value//$'\n'/\\n}"
  printf '%s' "$value"
}

all_match=true
first_entry=true
summary_entries=""

for name in core-csl core-bibtex csl bibtex; do
  left="$ROOT/tests/fixtures/format/${name}.txt"
  right="$RUBY_DIR/${name}.txt"

  left_exists=true
  if [[ ! -f "$left" ]]; then
    left_exists=false
  fi
  right_exists=true
  if [[ ! -f "$right" ]]; then
    right_exists=false
  fi

  left_lines=0
  if [[ "$left_exists" == true ]]; then
    left_lines="$(wc -l < "$left" | tr -d ' ')"
  fi
  right_lines=0
  if [[ "$right_exists" == true ]]; then
    right_lines="$(wc -l < "$right" | tr -d ' ')"
  fi

  diff_output=""
  if [[ "$left_exists" == true && "$right_exists" == true ]]; then
    diff_output="$(diff -u "$left" "$right" || true)"
  fi

  {
    echo "== diff: $name =="
    if [ -f "$left" ] && [ -f "$right" ]; then
      if [[ -n "$diff_output" ]]; then
        printf '%s\n' "$diff_output"
      fi
    else
      echo "missing fixture(s) for $name"
    fi
    echo ""
  } >> "$REPORT_PATH"

  removed_raw="$(printf '%s\n' "$diff_output" | grep -c '^-' || true)"
  removed_header="$(printf '%s\n' "$diff_output" | grep -c '^--- ' || true)"
  removed="$((removed_raw - removed_header))"

  added_raw="$(printf '%s\n' "$diff_output" | grep -c '^+' || true)"
  added_header="$(printf '%s\n' "$diff_output" | grep -c '^+++ ' || true)"
  added="$((added_raw - added_header))"

  hunks="$(printf '%s\n' "$diff_output" | grep -c '^@@ ' || true)"
  diff_lines="$((added + removed))"

  matches=false
  if [[ "$left_exists" == true && "$right_exists" == true && -z "$diff_output" ]]; then
    matches=true
  fi
  if [[ "$matches" != true ]]; then
    all_match=false
  fi

  entry=$(
    cat <<EOF
{
  "name": "$(json_escape "$name")",
  "left": "$(json_escape "$left")",
  "right": "$(json_escape "$right")",
  "left_exists": $left_exists,
  "right_exists": $right_exists,
  "left_lines": $left_lines,
  "right_lines": $right_lines,
  "added_lines": $added,
  "removed_lines": $removed,
  "hunks": $hunks,
  "diff_lines": $diff_lines,
  "matches": $matches
}
EOF
  )
  if [[ "$first_entry" == true ]]; then
    summary_entries="$entry"
    first_entry=false
  else
    summary_entries="${summary_entries},
${entry}"
  fi
done

cat > "$SUMMARY_PATH" <<EOF
{
  "generated": "${GENERATED_AT}",
  "all_match": ${all_match},
  "comparisons": [
${summary_entries}
  ]
}
EOF

echo "wrote $REPORT_PATH"
echo "wrote $SUMMARY_PATH"

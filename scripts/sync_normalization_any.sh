#!/usr/bin/env bash
set -euo pipefail

REPO_URL=${ANYSTYLE_DATA_REPO:-https://github.com/inukshuk/anystyle-data}
TARGET_DIR=${ANYSTYLE_DATA_DIR:-tmp/anystyle-data}
OUTPUT_DIR=${NORMALIZATION_OUTPUT_DIR:-tests/fixtures/normalization-any}

if [ ! -d "$TARGET_DIR/.git" ]; then
  git clone --depth 1 "$REPO_URL" "$TARGET_DIR"
else
  git -C "$TARGET_DIR" pull --ff-only
fi

SOURCE_DIR="$TARGET_DIR/res"
if ! compgen -G "$SOURCE_DIR/*abbrev*.txt" > /dev/null && \
   ! compgen -G "$SOURCE_DIR/*locale*.txt" > /dev/null; then
  echo "no normalization assets found in $SOURCE_DIR" >&2
  exit 0
fi

cargo run --quiet -- normalization-sync \
  --source-dir "$SOURCE_DIR" \
  --output-dir "$OUTPUT_DIR"

echo "synced normalization assets to $OUTPUT_DIR"

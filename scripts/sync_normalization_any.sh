#!/usr/bin/env bash
set -euo pipefail

REPO_URL=${ANYSTYLE_DATA_REPO:-https://github.com/inukshuk/anystyle-data}
TARGET_DIR=${ANYSTYLE_DATA_DIR:-tmp/anystyle-data}
OUTPUT_DIR=${NORMALIZATION_OUTPUT_DIR:-tests/fixtures/normalization-any}
NORMALIZATION_REPO=${ANYSTYLE_NORMALIZATION_REPO:-https://github.com/inukshuk/anystyle-data}
NORMALIZATION_DIR=${ANYSTYLE_NORMALIZATION_DIR:-}
NORMALIZATION_SUBDIR=${ANYSTYLE_NORMALIZATION_SUBDIR:-}

sync_from_dir() {
  local source="$1"
  if compgen -G "$source/*abbrev*.txt" > /dev/null || \
     compgen -G "$source/*locale*.txt" > /dev/null; then
    cargo run --quiet -- normalization-sync \
      --source-dir "$source" \
      --output-dir "$OUTPUT_DIR"
    echo "synced normalization assets from $source"
    return 0
  fi
  return 1
}

if [ -n "$NORMALIZATION_DIR" ]; then
  if sync_from_dir "$NORMALIZATION_DIR"; then
    exit 0
  fi
  echo "no normalization assets found in $NORMALIZATION_DIR" >&2
fi

if [ -n "$NORMALIZATION_REPO" ]; then
  if [[ "$NORMALIZATION_REPO" == http* ]]; then
    if [ ! -d "$TARGET_DIR-normalization/.git" ]; then
      git clone --depth 1 "$NORMALIZATION_REPO" "$TARGET_DIR-normalization"
    else
      git -C "$TARGET_DIR-normalization" pull --ff-only
    fi
    NORMALIZATION_REPO="$TARGET_DIR-normalization"
  fi
  if [ -n "$NORMALIZATION_SUBDIR" ]; then
    NORMALIZATION_REPO="$NORMALIZATION_REPO/$NORMALIZATION_SUBDIR"
  fi
  if sync_from_dir "$NORMALIZATION_REPO"; then
    exit 0
  fi
  echo "no normalization assets found in $NORMALIZATION_REPO" >&2
fi

if [ ! -d "$TARGET_DIR/.git" ]; then
  git clone --depth 1 "$REPO_URL" "$TARGET_DIR"
else
  git -C "$TARGET_DIR" pull --ff-only
fi

SOURCE_DIR="$TARGET_DIR/res"
if ! sync_from_dir "$SOURCE_DIR"; then
  echo "no normalization assets found in $SOURCE_DIR" >&2
fi

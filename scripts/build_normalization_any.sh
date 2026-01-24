#!/usr/bin/env bash
set -euo pipefail

SOURCE_DIR=${1:?source dir required}
OUTPUT_DIR=${2:?output dir required}

mkdir -p "$OUTPUT_DIR"

journal_out="$OUTPUT_DIR/journal-abbrev.txt"
publisher_out="$OUTPUT_DIR/publisher-abbrev.txt"
container_out="$OUTPUT_DIR/container-abbrev.txt"
language_out="$OUTPUT_DIR/language-locale.txt"
script_out="$OUTPUT_DIR/script-locale.txt"

: > "$journal_out"
: > "$publisher_out"
: > "$container_out"

if compgen -G "$SOURCE_DIR/journals_*.txt" > /dev/null; then
  cat "$SOURCE_DIR"/journals_*.txt | \
    awk 'NF {print $0 "\t" $0}' >> "$journal_out"
fi

if compgen -G "$SOURCE_DIR/publishers_*.txt" > /dev/null; then
  cat "$SOURCE_DIR"/publishers_*.txt | \
    awk 'NF {print $0 "\t" $0}' >> "$publisher_out"
fi

if [ -s "$journal_out" ]; then
  cp "$journal_out" "$container_out"
fi

cat <<'LANG' > "$language_out"
en	en-US
fr	fr-FR
LANG

cat <<'SCRIPT' > "$script_out"
Latin	Latn
Cyrillic	Cyrl
SCRIPT


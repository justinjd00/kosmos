#!/usr/bin/env bash
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
work="$here/.test"

modules=(syntax algebra integrate solve series)

rm -rf "$work"
mkdir -p "$work"
for m in "${modules[@]}"; do cp "$here/src/$m.ml" "$work/"; done
cp "$here/test/run.ml" "$work/"
cd "$work"

ocamlfind ocamlopt -w +a-4-9-40-41-42-44-45-70 \
  "${modules[@]/%/.ml}" run.ml -o run

./run

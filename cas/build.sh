#!/usr/bin/env bash
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
out="${1:-$here/../web/public/cas.js}"
work="$here/.build"

modules=(syntax algebra integrate solve series)

rm -rf "$work"
mkdir -p "$work"
cp "$here/src/"*.ml "$work/"
cd "$work"

ocamlfind ocamlc -package js_of_ocaml -package js_of_ocaml-ppx -linkpkg \
  -w +a-4-9-40-41-42-44-45-70 \
  "${modules[@]/%/.ml}" main.ml -o cas.byte

js_of_ocaml --opt 3 --target-env browser --no-source-map cas.byte -o cas.js

mkdir -p "$(dirname "$out")"
cp cas.js "$out"
echo "wrote $out ($(wc -c < cas.js) bytes)"

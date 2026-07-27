#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
WRITER="$ROOT/scripts/write-examples-index.sh"
TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT

GALLERY="$TMP/gallery.json"
cat > "$GALLERY" <<'JSON'
{
  "examples": [
    {
      "id": "zeta",
      "title": "<Zeta & \"quoted\">",
      "summary": "<script>alert('x')</script>",
      "checks": [{"cue":"0 < 1","expect":"A & B \"quoted\" 'single'"}]
    },
    {
      "id": "alpha",
      "title": "Alpha title",
      "summary": "Alpha summary",
      "checks": [{"cue":"Artifact","expect":"The output exists."}]
    }
  ]
}
JSON

make_example() {
  local output=$1 name=$2 artifact=$3
  mkdir -p "$output/$name/rendered" "$output/$name/project"
  : > "$output/$name/rendered/$artifact"
}

FULL="$TMP/full"
mkdir -p "$FULL"
make_example "$FULL" alpha tone.wav
make_example "$FULL" zeta output.txt
bash "$WRITER" "$FULL" 2 "$GALLERY"

[[ $(rg -o 'data-example="[^"]+"' "$FULL/index.html" | wc -l | tr -d ' ') -eq 2 ]]
[[ $(rg -o 'data-example="[^"]+"' "$FULL/index.html" | sed -n '1p') == 'data-example="zeta"' ]]
rg -F '&lt;Zeta &amp; &quot;quoted&quot;&gt;' "$FULL/index.html" >/dev/null
rg -F '&lt;script&gt;alert(&apos;x&apos;)&lt;/script&gt;' "$FULL/index.html" >/dev/null
rg -F '0 &lt; 1' "$FULL/index.html" >/dev/null
rg -F 'A &amp; B &quot;quoted&quot; &apos;single&apos;' "$FULL/index.html" >/dev/null
rg -F '<div class="output-name">tone.wav</div>' "$FULL/index.html" >/dev/null
if rg -F '<script>alert' "$FULL/index.html" >/dev/null; then
  echo "example index emitted unescaped presentation HTML" >&2
  exit 1
fi

SUBSET="$TMP/subset"
mkdir -p "$SUBSET"
make_example "$SUBSET" alpha preview.mp4
bash "$WRITER" "$SUBSET" 1 "$GALLERY"
rg -F 'data-example="alpha"' "$SUBSET/index.html" >/dev/null
if rg -F 'data-example="zeta"' "$SUBSET/index.html" >/dev/null; then
  echo "example index emitted an unbuilt gallery entry" >&2
  exit 1
fi

make_example "$FULL" rogue output.txt
if bash "$WRITER" "$FULL" 3 "$GALLERY" >/dev/null 2>&1; then
  echo "example index accepted an uncataloged output directory" >&2
  exit 1
fi

echo "example index contracts passed"

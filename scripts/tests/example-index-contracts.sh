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
      "title": "<示例 & \"引号\">",
      "summary": "<script>alert('测试')</script>",
      "checks": [{"cue":"0 < 1","expect":"甲 & 乙 \"双引号\" '单引号'"}]
    },
    {
      "id": "alpha",
      "title": "阿尔法示例",
      "summary": "阿尔法摘要",
      "checks": [{"cue":"成品","expect":"输出文件存在。"}]
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
rg -F '<html lang="zh-CN">' "$FULL/index.html" >/dev/null
rg -F '<h1>VEAC 示例集</h1>' "$FULL/index.html" >/dev/null
rg -F '<h3>验收要点</h3>' "$FULL/index.html" >/dev/null
rg -F '&lt;示例 &amp; &quot;引号&quot;&gt;' "$FULL/index.html" >/dev/null
rg -F '&lt;script&gt;alert(&apos;测试&apos;)&lt;/script&gt;' "$FULL/index.html" >/dev/null
rg -F '0 &lt; 1' "$FULL/index.html" >/dev/null
rg -F '甲 &amp; 乙 &quot;双引号&quot; &apos;单引号&apos;' "$FULL/index.html" >/dev/null
rg -F '<div class="output-name">tone.wav</div>' "$FULL/index.html" >/dev/null
rg -F '>打开 output.txt</a>' "$FULL/index.html" >/dev/null
rg -F '>源码</a>' "$FULL/index.html" >/dev/null
rg -F '>中间表示</a>' "$FULL/index.html" >/dev/null
if rg -F 'What to verify' "$FULL/index.html" >/dev/null; then
  echo "example index emitted English presentation chrome" >&2
  exit 1
fi
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

#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
WRITER="$ROOT/scripts/write-examples-index.sh"
source "$ROOT/scripts/tests/example-preview-layout-fixtures.sh"
TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT

GALLERY="$TMP/gallery.json"
cat > "$GALLERY" <<'JSON'
{
  "targets": [
    {"id":"zeta","expected_artifacts":[
      {"kind":"source_revision"},{"kind":"source_index"},
      {"kind":"source_edit_batch"},{"kind":"source_edit_outcome"},
      {"kind":"edit_batch"},{"kind":"edit_outcome"},
      {"kind":"edit_replay_outcome"},{"kind":"probe_snapshot"},
      {"kind":"delivery","logical_key":"preview","artifacts":[]}]},
    {"id":"alpha","expected_artifacts":[
      {"kind":"delivery","logical_key":"preview","artifacts":[]}]}
  ],
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
  local name=$2 artifact=$3 entry="$1/$2"
  prepare_preview_fixture_dirs "$entry"
  : > "$entry/rendered/$artifact"
  cat >"$entry/project/project.veac.json" <<JSON
{"project":{"id":"prj_$name","authorship":{"entity":{"logical_path":["$name"],"events":[]},
"multicam_groups":[],"annotations":[],"deliveries":[{"render_config_id":"out_4a4f4ce03f87",
"entity":{"logical_path":["$name","preview"],"events":[]}}]},
"render_configs":[{"id":"out_4a4f4ce03f87",
"deliverables":[{"id":"dlv_92f3","target":{"type":"file","name":"$artifact"}}]}]}}
JSON
  mirror_fixture_preview_canonical "$entry"
  printf '{"output":{"render_config_id":"out_4a4f4ce03f87"}}\n' \
    >"$entry/plans/preview/out_4a4f4ce03f87.json"
  printf 'fixture build\n' >"$entry/build.log"
}

FULL="$TMP/full"
mkdir -p "$FULL"
make_example "$FULL" alpha tone.wav
make_example "$FULL" zeta output.txt
printf 'module {}\n' >"$FULL/alpha/project/brand.veac"
for evidence in source.revision.json source.index.json source-edit.json \
    source-edit.outcome.json; do
  printf '{}\n' >"$FULL/zeta/project/$evidence"
  printf '{}\n' >"$FULL/alpha/project/$evidence"
done
for evidence in edit.batch.json edit.outcome.json edit.replay.outcome.json \
    probe.snapshot.json; do
  printf '{}\n' >"$FULL/zeta/project/$evidence"
  printf '{}\n' >"$FULL/alpha/project/$evidence"
done
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
rg -F '>创作源码</a>' "$FULL/index.html" >/dev/null
rg -F '>模块源码 brand.veac</a>' "$FULL/index.html" >/dev/null
rg -F 'href="zeta/project/source.revision.json">源码版本</a>' "$FULL/index.html" >/dev/null
rg -F 'href="zeta/project/source.index.json">源码索引</a>' "$FULL/index.html" >/dev/null
rg -F 'href="zeta/project/source-edit.json">源码编辑批次</a>' "$FULL/index.html" >/dev/null
rg -F 'href="zeta/project/source-edit.outcome.json">源码编辑预演结果</a>' \
  "$FULL/index.html" >/dev/null
rg -F 'href="zeta/project/edit.batch.json">规范编辑批次</a>' "$FULL/index.html" >/dev/null
rg -F 'href="zeta/project/edit.outcome.json">规范编辑结果</a>' "$FULL/index.html" >/dev/null
rg -F 'href="zeta/project/edit.replay.outcome.json">幂等重放结果</a>' \
  "$FULL/index.html" >/dev/null
rg -F 'href="zeta/project/probe.snapshot.json">素材探测快照</a>' "$FULL/index.html" >/dev/null
for evidence in source.revision.json source.index.json source-edit.json \
    source-edit.outcome.json; do
  if rg -F "href=\"alpha/project/$evidence\"" "$FULL/index.html" >/dev/null; then
    echo "example index exposed undeclared source edit evidence: $evidence" >&2
    exit 1
  fi
done
for evidence in edit.batch.json edit.outcome.json edit.replay.outcome.json \
    probe.snapshot.json; do
  if rg -F "href=\"alpha/project/$evidence\"" "$FULL/index.html" >/dev/null; then
    echo "example index exposed undeclared workflow evidence: $evidence" >&2
    exit 1
  fi
done
if rg -F '>预览适配源码</a>' "$FULL/index.html" >/dev/null; then
  echo "example index exposed a second source of truth" >&2
  exit 1
fi
rg -F '>创作 canonical IR</a>' "$FULL/index.html" >/dev/null
rg -F '>预览派生 IR</a>' "$FULL/index.html" >/dev/null
rg -F '>预览计划 out_4a4f4ce03f87</a>' "$FULL/index.html" >/dev/null
if rg -F '/plan.json' "$FULL/index.html" >/dev/null; then
  echo "example index linked a legacy ambiguous plan" >&2
  exit 1
fi
if rg -F 'What to verify' "$FULL/index.html" >/dev/null; then
  echo "example index emitted English presentation chrome" >&2
  exit 1
fi
if rg -F '<script>alert' "$FULL/index.html" >/dev/null; then
  echo "example index emitted unescaped presentation HTML" >&2
  exit 1
fi

REAL="$TMP/real"
mkdir -p "$REAL"
make_example "$REAL" agentsmesh-intro-15s preview.mp4
make_example "$REAL" executable-local-image preview.mp4
bash "$WRITER" "$REAL" 2 "$ROOT/examples/catalog/gallery.json"
rg -F '<h2>智能体协作网片头</h2>' "$REAL/index.html" >/dev/null
rg -F '0-1.17 秒 - 品牌逐字入场' "$REAL/index.html" >/dev/null
rg -F '白色“智能体协作网”' "$REAL/index.html" >/dev/null
rg -F '64×36 创作画布' "$REAL/index.html" >/dev/null
rg -F '低分辨率是本示例验证原生图片资源和类型化变换机制的有意设计' \
  "$REAL/index.html" >/dev/null

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

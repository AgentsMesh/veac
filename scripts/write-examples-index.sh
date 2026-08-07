#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
source "$SCRIPT_DIR/example-preview-layout.sh"
source "$SCRIPT_DIR/example-preview-provenance.sh"
source "$SCRIPT_DIR/example-source-edit-evidence.sh"
source "$SCRIPT_DIR/example-preview-delivery.sh"
source "$SCRIPT_DIR/example-workflow-evidence.sh"

OUTPUT=${1:?output directory is required}
EXPECTED=${2:?expected example count is required}
GALLERY=${3:?gallery catalog is required}
TEMP="$OUTPUT/index.html.tmp"

fail() {
  echo "examples index: $*" >&2
  exit 1
}

[[ -f "$GALLERY" ]] || fail "missing gallery catalog: $GALLERY"
[[ "$EXPECTED" =~ ^[0-9]+$ ]] || fail "expected count must be an integer"

actual=$(find "$OUTPUT" -mindepth 1 -maxdepth 1 -type d | wc -l | tr -d ' ')
[[ "$actual" -eq "$EXPECTED" ]] || fail "expected $EXPECTED example directories, found $actual"

cat > "$TEMP" <<'HTML'
<!doctype html><html lang="zh-CN"><head><meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>VEAC 示例集</title><style>
:root{color-scheme:light;background:#f3f5f4;color:#17211c;font-family:Inter,ui-sans-serif,system-ui,sans-serif}
body{margin:0}.page-header{max-width:1440px;margin:auto;padding:32px 24px 20px}.page-header h1{font-size:30px;margin:0 0 8px;letter-spacing:0}.page-header p{margin:0;color:#526159;max-width:760px;line-height:1.5}
main{display:grid;grid-template-columns:repeat(auto-fit,minmax(min(100%,360px),1fr));gap:18px;max-width:1440px;margin:auto;padding:0 24px 40px}
article{background:#fff;border:1px solid #d8dfdb;border-radius:6px;overflow:hidden;min-width:0;box-shadow:0 3px 14px #17211c12}
.copy{padding:20px;border-bottom:1px solid #e2e7e4}.example-id{margin:0 0 8px;color:#66736c;font-size:12px;overflow-wrap:anywhere}.example-id code{font:inherit}
h2{font-size:20px;line-height:1.25;margin:0 0 6px;letter-spacing:0}.summary{margin:0;color:#46544c;line-height:1.5;overflow-wrap:anywhere}
h3{font-size:13px;text-transform:uppercase;color:#2b6a4c;margin:18px 0 8px;letter-spacing:0}.checks{list-style:none;padding:0;margin:0;display:grid;gap:9px}
.checks li{display:grid;gap:2px;line-height:1.4;overflow-wrap:anywhere}.cue{font-size:12px;font-weight:700;color:#2b6a4c}.expect{font-size:13px;color:#35443c}
.outputs{display:grid;gap:1px;background:#e2e7e4}.output{background:#fff;padding-bottom:12px}.output-name{font-size:12px;font-weight:700;color:#526159;padding:10px 14px 8px;overflow-wrap:anywhere}
video,img,audio{display:block;width:100%;background:#101713}video,img{aspect-ratio:16/9;object-fit:contain}audio{box-sizing:border-box;padding:10px 14px}
.links{display:flex;gap:8px;flex-wrap:wrap;padding:14px 20px 18px}.links a,.artifact{font-size:12px;color:#1f5f43;text-decoration:none;border-bottom:1px solid #9db8a9;overflow-wrap:anywhere}.artifact{display:inline-block;margin:0 14px 12px}
@media(max-width:520px){.page-header{padding:24px 16px 16px}main{padding:0 16px 28px}.copy{padding:18px}.links{padding:12px 18px 16px}}
</style></head><body><header class="page-header"><h1>VEAC 示例集</h1>
<p>每个示例都会说明所展示的编辑机制，以及能够确认渲染成功的可观察结果。</p></header><main>
HTML

published=0
while IFS= read -r name; do
  dir="$OUTPUT/$name"
  [[ -d "$dir" ]] || continue
  metadata=$(jq -ce --arg id "$name" '.examples[] | select(.id == $id)' "$GALLERY") \
    || fail "missing presentation for $name"
  target=$(jq -ce --arg id "$name" '.targets[] | select(.id == $id)' "$GALLERY") \
    || fail "missing build target for $name"
  verify_example_preview_layout "$dir" || fail "invalid preview layout for $name"
  require_preview_regular_file "$dir/build.log" "build log" || exit 1
  primary_delivery=$(jq -er '
    [.expected_artifacts[] | select(.kind == "delivery") | .logical_key]
    | if length == 1 then .[0] else error("expected one delivery") end
  ' <<<"$target") || fail "missing primary delivery for $name"
  primary_config=$(delivery_config_id "$(example_preview_canonical "$dir")" \
    "$primary_delivery") || fail "cannot resolve primary delivery for $name"
  primary_plan=$(example_preview_plan "$dir" "$primary_config") ||
    fail "unsafe primary preview config for $name"
  require_preview_regular_file "$primary_plan" "primary preview plan" || exit 1
  title=$(jq -r '.title | @html' <<<"$metadata")
  summary=$(jq -r '.summary | @html' <<<"$metadata")
  checks=$(jq -r '.checks[] | "<li><span class=\"cue\">\(.cue | @html)</span><span class=\"expect\">\(.expect | @html)</span></li>"' <<<"$metadata")
  name_html=$(jq -nr --arg value "$name" '$value | @html')
  {
    printf '<article data-example="%s"><div class="copy"><p class="example-id"><code>%s</code></p>' "$name_html" "$name_html"
    printf '<h2>%s</h2><p class="summary">%s</p><h3>验收要点</h3><ul class="checks">%s</ul></div>' "$title" "$summary" "$checks"
    printf '<div class="outputs">'
  } >> "$TEMP"
  while IFS= read -r artifact; do
    relative=${artifact#"$dir/rendered/"}
    relative_html=$(jq -nr --arg value "$relative" '$value | @html')
    printf '<div class="output"><div class="output-name">%s</div>' "$relative_html" >> "$TEMP"
    case "$artifact" in
      *.mp4|*.webm) printf '<video controls preload="metadata" src="%s/rendered/%s"></video>' "$name_html" "$relative_html" >> "$TEMP" ;;
      *.png|*.jpg|*.jpeg) printf '<img loading="lazy" src="%s/rendered/%s" alt="%s 输出画面">' "$name_html" "$relative_html" "$name_html" >> "$TEMP" ;;
      *.wav|*.mp3|*.m4a) printf '<audio controls preload="metadata" src="%s/rendered/%s"></audio>' "$name_html" "$relative_html" >> "$TEMP" ;;
      *) printf '<a class="artifact" href="%s/rendered/%s">打开 %s</a>' "$name_html" "$relative_html" "$relative_html" >> "$TEMP" ;;
    esac
    printf '</div>' >> "$TEMP"
  done < <(find "$dir/rendered" -type f | sort)
  {
    printf '</div><nav class="links"><a href="%s/project/main.veac">创作源码</a>' "$name_html"
    while IFS= read -r module; do
      relative=${module#"$dir/project/"}
      relative_html=$(jq -nr --arg value "$relative" '$value | @html')
      printf '<a href="%s/project/%s">模块源码 %s</a>' \
        "$name_html" "$relative_html" "$relative_html"
    done < <(find "$dir/project" -type f -name '*.veac' \
      ! -name 'main.veac' | sort)
    if source_edit_evidence_requested "$target"; then
      while IFS=$'\t' read -r file label; do
        require_preview_regular_file "$dir/project/$file" "$label" || exit 1
        printf '<a href="%s/project/%s">%s</a>' "$name_html" "$file" "$label"
      done <<'ARTIFACTS'
source.revision.json	源码版本
source.index.json	源码索引
source-edit.json	源码编辑批次
source-edit.outcome.json	源码编辑预演结果
ARTIFACTS
    fi
    if edit_evidence_requested "$target"; then
      while IFS=$'\t' read -r file label; do
        require_preview_regular_file "$dir/project/$file" "$label" || exit 1
        printf '<a href="%s/project/%s">%s</a>' "$name_html" "$file" "$label"
      done <<'ARTIFACTS'
edit.batch.json	规范编辑批次
edit.outcome.json	规范编辑结果
edit.replay.outcome.json	幂等重放结果
ARTIFACTS
    fi
    if probe_evidence_requested "$target"; then
      require_preview_regular_file "$dir/project/probe.snapshot.json" "素材探测快照" || exit 1
      printf '<a href="%s/project/probe.snapshot.json">素材探测快照</a>' "$name_html"
    fi
    printf '<a href="%s/project/project.veac.json">创作 canonical IR</a>' "$name_html"
    printf '<a href="%s/project/project.preview.veac.json">预览派生 IR</a>' "$name_html"
    while IFS= read -r config; do
      config_html=$(jq -nr --arg value "$config" '$value | @html')
      printf '<a href="%s/plans/preview/%s.json">预览计划 %s</a>' \
        "$name_html" "$config_html" "$config_html"
    done < <(jq -r '.project.render_configs[].id' "$(example_preview_canonical "$dir")" | sort)
    printf '<a href="%s/build.log">构建日志</a></nav></article>\n' "$name_html"
  } >> "$TEMP"
  published=$((published + 1))
done < <(jq -r '.examples[].id' "$GALLERY")

[[ "$published" -eq "$EXPECTED" ]] || fail "expected $EXPECTED cataloged examples, published $published"
printf '</main></body></html>\n' >> "$TEMP"
mv "$TEMP" "$OUTPUT/index.html"

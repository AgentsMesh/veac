#!/usr/bin/env bash

make_text_layout_project() {
  local dir=$1 authoring preview
  mkdir -p "$dir/project/assets"
  cp "$(find_preview_arabic_font)" "$dir/project/assets/preview-arabic-font.ttf"
  authoring=$(example_authoring_canonical "$dir")
  preview=$(example_preview_canonical "$dir")
  cat >"$authoring" <<'JSON'
{"project":{"materials":[],"sequences":[{"tracks":[{"clips":[
  {"id":"itm_paragraph","source":{"type":"text","text":"中文排版 · مرحبا · VEAC","style":{"font":{"type":"family","family":"Arial Unicode MS"},"fallback_fonts":[{"type":"family","family":"Arial"}]}},"visual":{}},
  {"id":"itm_vertical-label","source":{"style":{"layout":{"writing_mode":"vertical-rl"}}},"visual":{"placement":{"anchor":"top_right"},"transform":{"anchor":{"x":1,"y":0}}}},
  {"id":"itm_vertical-left-label","source":{"style":{"layout":{"writing_mode":"vertical-lr"}}},"visual":{"placement":{"anchor":"bottom_left"},"transform":{"anchor":{"x":0,"y":1}}}},
  {"id":"itm_path-label","source":{"style":{"path":{"start_offset":{"unit":"percent","value":50},"points":[{},{},{}]}}},"visual":{}}
]}]}]}}
JSON
  jq '
    .project.materials = [
      {"id":"med_preview-font","source":{"type":"file","uri":"assets/preview-font.ttf"}},
      {"id":"med_preview-arabic-font","source":{"type":"file","uri":"assets/preview-arabic-font.ttf"}}
    ] |
    (.project.sequences[0].tracks[0].clips[0].source.style.font) =
      {"type":"material","material_id":"med_preview-font"} |
    (.project.sequences[0].tracks[0].clips[0].source.style.fallback_fonts) =
      [{"type":"material","material_id":"med_preview-arabic-font"}]
  ' "$authoring" >"$preview"
}

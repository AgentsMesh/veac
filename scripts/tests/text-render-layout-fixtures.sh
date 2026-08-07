#!/usr/bin/env bash

make_text_layout_project() {
  local dir=$1 authoring preview
  mkdir -p "$dir/project/assets"
  cp "$(find_preview_font)" "$dir/project/assets/preview-font.ttf"
  cp "$(find_preview_arabic_font)" "$dir/project/assets/preview-arabic-font.ttf"
  authoring=$(example_authoring_canonical "$dir")
  preview=$(example_preview_canonical "$dir")
  cat >"$authoring" <<'JSON'
{"project":{"materials":[{"id":"authored-font","source":{"type":"file","uri":"assets/veac-example-zh.ttf"}}],"sequences":[{"tracks":[{"clips":[
  {"id":"generated-horizontal","authorship":{"logical_path":["text-layout","main","copy","horizontal"],"events":[]},"source":{"type":"text","text":"中文排版 / مرحبا / VEAC","style":{"font":{"type":"material","material_id":"authored-font"},"fallback_fonts":[],"layout":{"wrap":"character","overflow":"ellipsis"},"spans":[{"font":{"type":"family","family":"Noto Sans SC"}},{"font":{"type":"family","family":"Noto Sans Arabic"}}]}},"visual":{"placement":{"anchor":"top_left","inset":{"x":32,"y":32},"type":"anchor"}}},
  {"id":"generated-vertical-rl","authorship":{"logical_path":["text-layout","main","copy","vertical-rl"],"events":[]},"source":{"style":{"color":{"alpha":255,"blue":238,"green":211,"red":34},"layout":{"writing_mode":"vertical-rl","wrap":"none"}}},"visual":{"placement":{"anchor":"top_right","inset":{"x":48,"y":48},"type":"anchor"},"transform":{"anchor":{"x":1,"y":0}}}},
  {"id":"generated-vertical-lr","authorship":{"logical_path":["text-layout","main","copy","vertical-lr"],"events":[]},"source":{"style":{"color":{"alpha":255,"blue":91,"green":138,"red":255},"layout":{"writing_mode":"vertical-lr"}}},"visual":{"placement":{"anchor":"top_left","inset":{"x":48,"y":48},"type":"anchor"},"transform":{"anchor":{"x":0,"y":0}}}},
  {"id":"generated-word-wrap","authorship":{"logical_path":["text-layout","main","copy","word-wrap"],"events":[]},"source":{"style":{"layout":{"writing_mode":"horizontal-tb","wrap":"word","overflow":"clip"}}},"visual":{"placement":{"anchor":"bottom","inset":{"x":0,"y":32},"type":"anchor"}}},
  {"id":"generated-path","authorship":{"logical_path":["text-layout","main","copy","path"],"events":[]},"source":{"style":{"color":{"alpha":255,"blue":21,"green":204,"red":250},"layout":{"wrap":"none"},"path":{"start_offset":{"unit":"pixels","value":300},"points":[{},{},{}]}}},"visual":{}}
]}]}]}}
JSON
  jq '
    .project.materials += [
      {"id":"med_preview-font","source":{"type":"file","uri":"assets/preview-font.ttf"}},
      {"id":"med_preview-arabic-font","source":{"type":"file","uri":"assets/preview-arabic-font.ttf"}}
    ] |
    (.project.sequences[0].tracks[0].clips[0].source.style.spans[0].font) =
      {"type":"material","material_id":"med_preview-font"} |
    (.project.sequences[0].tracks[0].clips[0].source.style.spans[1].font) =
      {"type":"material","material_id":"med_preview-arabic-font"}
  ' "$authoring" >"$preview"
}

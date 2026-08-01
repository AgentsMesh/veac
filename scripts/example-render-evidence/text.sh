#!/usr/bin/env bash

text_stats() {
  local video=$1 time=$2 x=$3 y=$4 width=$5 height=$6 threshold=$7 mode=${8:-bright}
  ffmpeg -v error -ss "$time" -i "$video" -frames:v 1 \
    -vf "crop=${width}:${height}:${x}:${y},format=rgb24" -f rawvideo - 2>/dev/null |
    od -An -v -tu1 |
    awk -v width="$width" -v threshold="$threshold" -v mode="$mode" '
      {
        for (i = 1; i <= NF; i++) {
          channel = bytes % 3
          if (channel == 0) r = $i
          else if (channel == 1) g = $i
          else {
            b = $i
            hit = r + g + b >= threshold
            if (mode == "yellow") hit = r > 150 && g > 110 && b < 190 && r > b + 35
            if (mode == "cyan") hit = r < 140 && g > 100 && b > 130 && b > r + 40
            if (mode == "orange") hit = r > 150 && g > 55 && g < 190 && b < 150 && r > b + 50
            if (hit) {
              x = pixel % width; y = int(pixel / width)
              if (count == 0 || x < min_x) min_x = x
              if (count == 0 || x > max_x) max_x = x
              if (count == 0 || y < min_y) min_y = y
              if (count == 0 || y > max_y) max_y = y
              count++
            }
            pixel++
          }
          bytes++
        }
      }
      END {
        if (count == 0) print "0 0 0 0 0"
        else print count, max_x - min_x + 1, max_y - min_y + 1, min_x, min_y
      }'
}

text_expect_box() {
  local video=$1 time=$2 x=$3 y=$4 width=$5 height=$6 threshold=$7
  local min_pixels=$8 min_width=$9 min_height=${10} max_width=${11} max_height=${12} label=${13}
  local count box_width box_height min_x min_y
  read -r count box_width box_height min_x min_y < <(
    text_stats "$video" "$time" "$x" "$y" "$width" "$height" "$threshold"
  )
  if ((count < min_pixels || box_width < min_width || box_height < min_height ||
    (max_width > 0 && box_width > max_width) || (max_height > 0 && box_height > max_height))); then
    fail "$label bbox mismatch: pixels=$count size=${box_width}x${box_height} origin=${min_x},${min_y}"
  fi
}

text_expect_quiet() {
  local video=$1 time=$2 x=$3 y=$4 width=$5 height=$6 threshold=$7 max_pixels=$8 label=$9
  local count _rest
  read -r count _rest < <(text_stats "$video" "$time" "$x" "$y" "$width" "$height" "$threshold")
  ((count <= max_pixels)) || fail "$label should be empty, found $count signal pixels"
}

text_rgb() {
  local video=$1 time=$2 x=$3 y=$4
  ffmpeg -v error -ss "$time" -i "$video" -frames:v 1 \
    -vf "crop=1:1:${x}:${y},format=rgb24" -f rawvideo - 2>/dev/null | od -An -v -tu1
}

check_text_media() {
  local video=$1 duration=$2 width=$3 height=$4 label=$5
  require_file "$video"
  assert_stream_count "$video" v 1 "$label"
  assert_stream_count "$video" a 0 "$label"
  assert_stream_field "$video" v:0 width "$width" "$label"
  assert_stream_field "$video" v:0 height "$height" "$label"
  assert_duration_close "$video" "$duration" 0.25 "$label"
}

check_text_layout() {
  local dir=$1 video=$2 authoring preview fallback langs
  local paragraph arabic latin left_orange left_cyan right_cyan right_orange
  local path_count path_width path_height
  authoring=$(example_authoring_canonical "$dir")
  preview=$(example_preview_canonical "$dir")
  require_file "$authoring"
  require_file "$preview"
  jq -e '
    def clip($id): first(.project.sequences[].tracks[].clips[] | select(.id == $id));
    clip("itm_paragraph") as $p |
    $p.source.type == "text" and $p.source.text == "中文排版 · مرحبا · VEAC" and
    $p.source.style.font == {"type":"family","family":"Arial Unicode MS"} and
    $p.source.style.fallback_fonts == [{"type":"family","family":"Arial"}] and
    clip("itm_vertical-label").source.style.layout.writing_mode == "vertical-rl" and
    clip("itm_vertical-label").visual.placement.anchor == "top_right" and
    clip("itm_vertical-label").visual.transform.anchor == {"x":1,"y":0} and
    clip("itm_vertical-left-label").source.style.layout.writing_mode == "vertical-lr" and
    clip("itm_vertical-left-label").visual.placement.anchor == "bottom_left" and
    clip("itm_vertical-left-label").visual.transform.anchor == {"x":0,"y":1} and
    clip("itm_path-label").source.style.path.start_offset == {"unit":"percent","value":50} and
    (clip("itm_path-label").source.style.path.points | length) == 3
  ' "$authoring" >/dev/null || fail "text-layout authoring semantics contract failed"
  jq -e '
    def clip($id): first(.project.sequences[].tracks[].clips[] | select(.id == $id));
    def material($id): first(.project.materials[] | select(.id == $id));
    clip("itm_paragraph").source.style as $style |
    $style.font.material_id as $primary |
    first($style.fallback_fonts[] | select(.material_id != $primary)) as $fallback |
    material($primary).source.uri == "assets/preview-font.ttf" and
    material($fallback.material_id).source.uri == "assets/preview-arabic-font.ttf"
  ' "$preview" >/dev/null || fail "text-layout preview font adaptation contract failed"
  fallback="$dir/project/assets/preview-arabic-font.ttf"
  require_file "$fallback"
  command -v fc-query >/dev/null 2>&1 || fail "fc-query is required for Arabic font evidence"
  langs=$(fc-query -i 0 --format='%{lang}\n' "$fallback") || fail "Arabic fallback font cannot be queried"
  tr '|' '\n' <<<"$langs" | grep -qx ar || fail "text-layout fallback font does not cover Arabic"
  check_text_media "$video" 6 480 480 "text-layout"
  read -r paragraph _ < <(text_stats "$video" 1 0 0 480 480 0 cyan)
  read -r arabic _ < <(text_stats "$video" 1 0 0 480 480 0 yellow)
  read -r latin _ < <(text_stats "$video" 1 0 0 480 480 680 bright)
  read -r left_orange _ < <(text_stats "$video" 3 0 0 220 480 0 orange)
  read -r left_cyan _ < <(text_stats "$video" 3 0 0 220 480 0 cyan)
  read -r right_cyan _ < <(text_stats "$video" 3 260 0 220 480 0 cyan)
  read -r right_orange _ < <(text_stats "$video" 3 260 0 220 480 0 orange)
  ((paragraph >= 40 && arabic >= 40 && latin >= 20)) ||
    fail "text-layout paragraph scripts are missing: Chinese=$paragraph Arabic=$arabic Latin=$latin"
  ((left_orange >= 40 && right_cyan >= 40 && left_cyan <= 15 && right_orange <= 15)) ||
    fail "text-layout vertical blocks overlap or use the wrong side"
  read -r path_count path_width path_height _ _ < <(text_stats "$video" 5 0 0 480 480 0 yellow)
  ((path_count >= 40 && path_width >= 140 && path_height >= 50)) ||
    fail "text-layout path text is missing or remains horizontal"
}

check_caption_text() {
  local example_dir=$1 video=$2 first_count first_width second_count second_width sidecar expected canonical first_panel second_panel
  canonical="$example_dir/project/project.veac.json"; require_file "$canonical"
  jq -e '
    . as $root |
    def clip($id): first($root.project.sequences[].tracks[].clips[] | select(.id == $id));
    clip("itm_opening").source.style.background == {"color":{"alpha":170,"blue":0,"green":0,"red":0},"padding_pixels":12} and
    clip("itm_closing").source.style.background == null and all("itm_opening", "itm_closing";
      clip(.).visual.placement == {"anchor":"bottom","inset":{"x":0,"y":48},"type":"anchor"})
  ' "$canonical" >/dev/null || fail "caption items must use the declared bottom placement"
  check_text_media "$video" 6 480 270 "captions-and-sidecars"
  read -r first_count first_width _ _ _ < <(text_stats "$video" 1.5 0 170 480 100 500)
  read -r second_count second_width _ _ _ < <(text_stats "$video" 4.5 0 170 480 100 500)
  ((first_count >= 120 && second_count >= 120 && second_width > first_width + 20)) ||
    fail "caption cue regions are incorrect: first=${first_count}px/${first_width}w second=${second_count}px/${second_width}w"
  first_panel=$(region_color_count "$video" 1.5 '280:60:100:210' black)
  second_panel=$(region_color_count "$video" 4.5 '280:60:100:210' black)
  ((first_panel > second_panel + 500)) || fail "opening caption background is missing: first=$first_panel second=$second_panel"
  sidecar="$example_dir/rendered/captions.vtt"; require_file "$sidecar"
  expected=$'WEBVTT\n\n00:00:00.000 --> 00:00:03.000\n<v 旁白>字幕是时间线中的类型化源。</v>\n\n00:00:03.000 --> 00:00:06.000\n<v 旁白>伴随文件从类型化字幕图层中选择内容。</v>\n\n'
  cmp -s <(printf '%s' "$expected") "$sidecar" || fail "captions.vtt cues differ from the declared caption layer"
}

check_agentsmesh_intro() {
  local video=$1 early_count brand_count
  check_text_media "$video" 15 480 270 "agentsmesh-intro-15s"
  read -r early_count _ _ _ _ < <(text_stats "$video" 0.2 70 65 340 100 540)
  read -r brand_count _ _ _ _ < <(text_stats "$video" 1.5 70 65 340 100 540)
  ((brand_count > early_count + 80)) || fail "AgentsMesh brand reveal did not progress"
  text_expect_quiet "$video" 1.5 40 205 400 55 540 30 "intro caption before its cue"
  text_expect_box "$video" 3 40 205 400 55 540 100 100 7 400 55 "intro caption"
  text_expect_box "$video" 7 30 90 420 85 500 180 220 10 420 85 "intro promise"
  text_expect_quiet "$video" 13 40 205 400 55 540 30 "expired intro caption"
  text_expect_box "$video" 13 30 90 420 85 500 180 220 10 420 85 "persistent intro promise"
}

check_hello_world() {
  local video=$1 top_r top_g top_b bottom_r bottom_g bottom_b
  check_text_media "$video" 4 480 270 "hello-world"
  text_expect_box "$video" 2 70 90 340 90 560 180 100 15 340 90 "hello-world title"
  read -r top_r top_g top_b < <(text_rgb "$video" 2 12 12)
  read -r bottom_r bottom_g bottom_b < <(text_rgb "$video" 2 12 258)
  ((top_g > bottom_g + 12 && top_b > bottom_b + 15)) ||
    fail "hello-world vertical gradient is missing: top=$top_r,$top_g,$top_b bottom=$bottom_r,$bottom_g,$bottom_b"
}

check_text_render_contract() {
  local name=$1 example_dir=$2 video=$3
  case "$name" in
    text-layout) check_text_layout "$example_dir" "$video" ;;
    text-animation) check_text_animation "$video" ;;
    text-overlay) check_text_overlay "$example_dir" "$video" ;;
    captions-and-sidecars) check_caption_text "$example_dir" "$video" ;;
    agentsmesh-intro-15s) check_agentsmesh_intro "$video" ;;
    hello-world) check_hello_world "$video" ;;
    *) return 0 ;;
  esac
}

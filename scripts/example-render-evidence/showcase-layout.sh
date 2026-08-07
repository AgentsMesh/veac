#!/usr/bin/env bash

check_card_overlay_evidence() {
  local dir="$PREVIEW_ROOT/card-overlay"
  [[ -d $dir ]] || return 0
  local video="$dir/rendered/preview.mp4" canonical="$dir/project/project.veac.json"
  local top bottom left right center_r center_g center_b corner outside inside
  local shadow shadow_left shadow_right clear clear_left clear_right
  video_contract "$video" 3.5
  jq -e '
    def clip($key): first(.project.sequences[].tracks[].clips[] |
      select(.authorship.logical_path[-1] == $key));
    clip("panel") as $card |
    $card.visual.frame == {"fit":"fill","height":{"unit":"pixels","value":380},
      "width":{"unit":"pixels","value":720}} and
    $card.visual.placement == {"anchor":"center","inset":{"x":0,"y":0},"type":"anchor"} and
    $card.visual.opacity == {"type":"constant","value":0.92} and
    $card.visual.card == {"corner_radius_pixels":36,"shadow":{"blur_pixels":28,
      "color":{"alpha":255,"blue":0,"green":0,"red":0},"offset":{"x":0,"y":16},
      "opacity":0.38}} and
    $card.source.generator.shape.geometry.type == "rectangle" and
    $card.source.generator.shape.stroke.width_pixels == 14
  ' "$canonical" >/dev/null || fail "card-overlay canonical card contract failed"
  top=$(region_color_count "$video" 1.5 '240:8:120:60' cyan)
  bottom=$(region_color_count "$video" 1.5 '240:8:120:202' cyan)
  left=$(region_color_count "$video" 1.5 '8:120:101:75' cyan)
  right=$(region_color_count "$video" 1.5 '8:120:371:75' cyan)
  ((top > 40 && bottom > 40 && left > 20 && right > 20)) ||
    fail "card-overlay outline is clipped: top=$top bottom=$bottom left=$left right=$right"
  read -r center_r center_g center_b <<<"$(region_rgb "$video" 1.5 '40:40:220:115')"
  ((center_r > 205 && center_r < 245 && center_g > 205 && center_b > 205)) ||
    fail "card-overlay translucent surface has unexpected RGB: $center_r,$center_g,$center_b"
  corner=$(region_yavg "$video" 1.5 '4:4:105:64')
  outside=$(region_yavg "$video" 1.5 '4:4:95:64')
  inside=$(region_yavg "$video" 1.5 '4:4:114:73')
  awk -v corner="$corner" -v outside="$outside" -v inside="$inside" '
    BEGIN { delta=corner-outside; if (delta<0) delta=-delta; exit !(delta<20 && inside>corner+80) }
  ' ||
    fail "card-overlay rounded corner is not visible"
  shadow=$(region_yavg "$video" 1.5 '100:4:190:208')
  shadow_left=$(region_yavg "$video" 1.5 '60:4:20:208')
  shadow_right=$(region_yavg "$video" 1.5 '60:4:400:208')
  clear=$(region_yavg "$video" 1.5 '100:4:190:245')
  clear_left=$(region_yavg "$video" 1.5 '60:4:20:245')
  clear_right=$(region_yavg "$video" 1.5 '60:4:400:245')
  awk -v shadow="$shadow" -v left="$shadow_left" -v right="$shadow_right" \
    -v clear="$clear" -v clear_left="$clear_left" -v clear_right="$clear_right" '
    BEGIN {
      shadow_delta=clear-shadow
      background_delta=((clear_left+clear_right)-(left+right))/2
      exit !(shadow_delta-background_delta>5)
    }
  ' || fail "card-overlay downward shadow is missing"
}

check_template_fill_evidence() {
  local dir="$PREVIEW_ROOT/template-fill"
  [[ -d $dir ]] || return 0
  local video="$dir/rendered/preview.mp4" canonical="$dir/project/project.veac.json"
  local time white yspread uspread vspread
  video_contract "$video" 3.5
  jq -e '
    def clip($key): first(.project.sequences[].tracks[].clips[] |
      select(.authorship.logical_path[-1] == $key));
    clip("hero").replaceable as $hero |
    $hero.fill == "fit_duration" and $hero.kind == "video" and
    $hero.label == "主视觉媒体" and
    ($hero.min_source_duration | .timescale > 0 and .value / .timescale == 2) and
    clip("title").replaceable == {"fill":"fit_duration","kind":"text",
      "label":"主标题文本","min_source_duration":null} and
    clip("title").template_editable_text == true and
    clip("title").source.text == "可替换标题"
  ' "$canonical" >/dev/null || fail "template-fill canonical slot contract failed"
  assert_unique_frames "$video" 4 0.5 1.5 2.5 3.5
  for time in 0.5 2.0 3.5; do
    white=$(region_color_count "$video" "$time" '320:70:80:100' white)
    ((white > 90)) || fail "template-fill title is missing at ${time}s: white=$white"
  done
  read -r yspread uspread vspread <<<"$(region_yuv_spread "$video" 2.5 '480:270:0:0')"
  ((yspread > 100 && (uspread > 50 || vspread > 50))) ||
    fail "template-fill placeholder does not fill the canvas"
}

check_all_features_evidence() {
  local dir="$PREVIEW_ROOT/all-features"
  [[ -d $dir ]] || return 0
  local canonical="$dir/project/project.veac.json" video caption edge_navy panel_navy
  video=$(delivery_video_path "$dir" master all-features.mp4)
  video_contract "$video" 3.5
  jq -e '
    def clip($key): first(.project.sequences[].tracks[].clips[] |
      select(.authorship.logical_path[-1] == $key));
    clip("cue").source.text == "一种语言，一份类型化中间表示，一套渲染计划。" and
    clip("lower-third").visual.frame.height == {"unit":"pixels","value":180} and
    any(.project.relations[]; .kind.type == "group") and
    any(.project.relations[]; .kind.type == "av_link")
  ' "$canonical" >/dev/null || fail "all-features canonical integration contract failed"
  assert_unique_frames "$video" 4 0.5 1.5 2.5 3.5
  caption=$(region_color_count "$video" 1.5 '360:35:60:195' white)
  ((caption > 35)) || fail "all-features burned caption is missing at 1.5s"
  caption=$(region_color_count "$video" 3.5 '360:35:60:195' white)
  ((caption > 35)) || fail "all-features burned caption is missing at 3.5s"
  panel_navy=$(region_color_count "$video" 1.5 '360:60:60:175' navy)
  edge_navy=$(region_color_count "$video" 1.5 '45:60:0:175' navy)
  ((panel_navy > 9000 && edge_navy < 1800)) ||
    fail "all-features lower third is missing or covers the side margin"
}

#!/usr/bin/env bash

all_features_authoring_visual_contract() {
  local canonical=$1 shot panel caption music
  shot=$(canonical_clip_id "$canonical" shot); panel=$(canonical_clip_id "$canonical" lower-third)
  caption=$(canonical_clip_id "$canonical" cue); music=$(canonical_clip_id "$canonical" music-item)
  jq -e --arg shot "$shot" --arg panel "$panel" --arg caption "$caption" --arg music "$music" '
    def seconds: .value/.timescale;
    def clip($id): first(.project.sequences[].tracks[].clips[] | select(.id == $id));
    clip($shot) as $shot | clip($panel) as $panel | clip($caption) as $caption |
    ($shot.record_range.start|seconds) == 0 and ($shot.record_range.duration|seconds) == 8 and
    ($shot.source_mapping.time_map.source_start|seconds) == 2 and
    $shot.source_mapping.time_map.rate == {"numerator":1,"denominator":1} and
    $shot.visual.color_pipeline as $grade |
    $grade.input == {"matrix":"bt709","primaries":"bt709","range":"limited","transfer":"bt709"} and
    $grade.working == {"matrix":"rgb","primaries":"bt709","range":"full","transfer":"linear"} and
    $grade.output == $grade.input and
    $grade.stages == [{"type":"basic","adjustment":{"exposure_stops":0.1,"fade":0.01,
      "highlights":-0.04,"shadows":0.06,"temperature_kelvin":6600,"tint":0}}] and
    ($panel.record_range.start|seconds) == 1 and ($panel.record_range.duration|seconds) == 5 and
    $panel.source.generator.color == {"alpha":221,"blue":94,"green":4,"red":3} and
    $panel.visual.frame == {"fit":"fill","height":{"unit":"pixels","value":180},
      "width":{"unit":"pixels","value":1500}} and
    $panel.visual.placement == {"anchor":"bottom","inset":{"x":0,"y":80},"type":"anchor"} and
    $panel.visual.opacity == {"type":"constant","value":0.95} and
    ($caption.record_range.start|seconds) == 0 and ($caption.record_range.duration|seconds) == 8 and
    $caption.source.text == "一种语言，一份类型化中间表示，一套渲染计划。" and
    $caption.source.style.background == {"color":
      {"alpha":204,"blue":0,"green":0,"red":0},"padding_pixels":20} and
    $caption.source.style.font_weight == "bold" and $caption.source.style.size_pixels == 64 and $caption.source.style.layout.box_width_pixels == 1500 and $caption.source.style.layout.box_height_pixels == 140 and $caption.source.style.layout.overflow == "clip" and
    $caption.visual.placement == {"type":"absolute","position":{"x":{"unit":"pixels","value":960},"y":{"unit":"pixels","value":850}}} and $caption.visual.frame == {"fit":"contain","height":{"unit":"pixels","value":140},"width":{"unit":"pixels","value":1500}} and
    any(.project.relations[]; .kind.type == "group" and
      ([.kind.members[].item_id] | sort) == ([$shot.id,$music] | sort)) and
    any(.project.relations[]; .kind.type == "av_link" and
      .kind.video.item_id == $shot.id and [.kind.audio[].item_id] == [$music])
  ' "$canonical" >/dev/null || fail "all-features authoring visual contract failed"
}

all_features_preview_visual_contract() {
  local canonical=$1 config shot panel caption
  config=$(delivery_config_id "$canonical" master)
  shot=$(canonical_clip_id "$canonical" shot); panel=$(canonical_clip_id "$canonical" lower-third)
  caption=$(canonical_clip_id "$canonical" cue)
  jq -e --arg config "$config" --arg shot "$shot" --arg panel "$panel" --arg caption "$caption" '
    def seconds: .value/.timescale;
    def clip($id): first(.project.sequences[].tracks[].clips[] | select(.id == $id));
    first(.project.render_configs[] | select(.id == $config)) as $out |
    (clip($shot).record_range.duration|seconds) == 8 and
    (clip($panel).record_range.start|seconds) == 1 and
    (clip($panel).record_range.duration|seconds) == 5 and
    (clip($caption).record_range.duration|seconds) == 8 and
    clip($caption).source.style.background == {"color":
      {"alpha":204,"blue":0,"green":0,"red":0},"padding_pixels":20} and
    clip($caption).source.style.font_weight == "bold" and clip($caption).source.style.size_pixels == 64 and clip($caption).source.style.layout.box_width_pixels == 1500 and clip($caption).source.style.layout.box_height_pixels == 140 and clip($caption).source.style.layout.overflow == "clip" and
    clip($caption).visual.placement == {"type":"absolute","position":{"x":{"unit":"pixels","value":960},"y":{"unit":"pixels","value":850}}} and clip($caption).visual.frame == {"fit":"contain","height":{"unit":"pixels","value":140},"width":{"unit":"pixels","value":1500}} and
    $out.raster == {"captions":"burn_in","frame_rate":{"denominator":1,"numerator":12},
      "height":270,"width":480} and
    first($out.deliverables[] | select(.kind.type == "video" and
      .target.name == "all-features.mp4")) as $video |
    $video.target == {"name":"all-features.mp4","type":"file"} and
    $video.kind.settings.video.codec == "h264" and
    $video.kind.settings.video.pixel_format == "yuv420p" and
    $video.kind.settings.audio == {"channels":2,"codec":"aac","sample_rate":48000} and
    $video.kind.settings.hardware == {"type":"software"}
  ' "$canonical" >/dev/null || fail "all-features preview visual contract failed"
}

all_features_plan_visual_contract() {
  local plan=$1 identity=$2 config sequence shot caption
  config=$(delivery_config_id "$identity" master); sequence=$(canonical_sequence_id "$identity" main)
  shot=$(canonical_clip_id "$identity" shot); caption=$(canonical_clip_id "$identity" cue)
  jq -e --arg config "$config" --arg sequence "$sequence" --arg shot "$shot" --arg caption "$caption" '
    def seconds: .value/.timescale;
    def clip($id): first(.sequences[].tracks[].clips[] | select(.id == $id));
    .output.render_config_id == $config and .output.sequence_id == $sequence and
    .output.raster == {"captions":"burn_in","frame_rate":{"denominator":1,"numerator":12},
      "height":270,"width":480} and
    first(.output.deliverables[] | select(.kind.type == "video" and
      .target.name == "all-features.mp4")).target ==
      {"name":"all-features.mp4","type":"file"} and
    (first(.sequences[] | select(.id == $sequence)).duration|seconds) == 8 and
    (clip($shot).source_mapping.time_map.source_range_per_repeat.start|seconds) == 2 and
    (clip($shot).source_mapping.time_map.source_range_per_repeat.duration|seconds) == 8 and
    clip($shot).visual.color_pipeline.stages[0].type == "basic" and
    clip($caption).source.type == "caption" and
    clip($caption).source.content.text ==
      "一种语言，一份类型化中间表示，一套渲染计划。" and
    clip($caption).source.content.presentation.type == "styled" and
    clip($caption).source.content.presentation.style.background == {"color":
      {"alpha":204,"blue":0,"green":0,"red":0},"padding_pixels":20} and clip($caption).source.content.presentation.style.font_weight == "bold" and clip($caption).source.content.presentation.style.size_pixels == 64 and clip($caption).source.content.presentation.style.layout.box_width_pixels == 1500 and clip($caption).source.content.presentation.style.layout.box_height_pixels == 140 and clip($caption).source.content.presentation.style.layout.overflow == "clip" and clip($caption).visual.placement == {"type":"absolute","position":{"x":{"unit":"pixels","value":960},"y":{"unit":"pixels","value":850}}} and clip($caption).visual.frame == {"fit":"contain","height":{"unit":"pixels","value":140},"width":{"unit":"pixels","value":1500}}
  ' "$plan" >/dev/null || fail "all-features preview plan visual contract failed"
}

all_features_mapping_score() {
  local render=$1 record_time=$2 source=$3 source_time=$4
  ffmpeg -nostdin -hide_banner -loglevel info -ss "$record_time" -i "$render" \
    -ss "$source_time" -i "$source" -frames:v 1 -filter_complex \
    '[0:v]crop=iw:ih/2:0:0,scale=160:45:flags=area[a];
     [1:v]crop=iw:ih/2:0:0,scale=160:45:flags=area[b];[a][b]psnr=stats_file=-' \
    -f null - 2>&1 | awk -F'psnr_avg:' \
    '/psnr_avg:/ { split($2,a," "); print (a[1] == "inf" ? 100 : a[1]); exit }'
}

all_features_assert_mapping() {
  local video=$1 record=$2 source=$3 expected=$4 wrong=$5 label=$6
  local expected_score wrong_score
  expected_score=$(all_features_mapping_score "$video" "$record" "$source" "$expected")
  wrong_score=$(all_features_mapping_score "$video" "$record" "$source" "$wrong")
  [[ -n $expected_score && -n $wrong_score ]] || fail "$label: PSNR unavailable"
  awk -v expected="$expected_score" -v wrong="$wrong_score" \
    'BEGIN { exit !(expected >= wrong + 1.5) }' ||
    fail "$label: expected=$expected_score wrong=$wrong_score"
}

all_features_assert_grade() {
  local video=$1 source=$2 rendered_rgb source_rgb delta
  rendered_rgb=$(region_rgb "$video" 0.5 'iw/3:ih/5:iw/12:ih/12')
  source_rgb=$(region_rgb "$source" 2.5 'iw/3:ih/5:iw/12:ih/12')
  read -r rr rg rb <<<"$rendered_rgb"
  read -r sr sg sb <<<"$source_rgb"
  delta=$(( ${rr:-0} - ${sr:-0} )); ((delta < 0)) && delta=$((-delta))
  local part=$(( ${rg:-0} - ${sg:-0} )); ((part < 0)) && part=$((-part)); delta=$((delta + part))
  part=$(( ${rb:-0} - ${sb:-0} )); ((part < 0)) && part=$((-part)); delta=$((delta + part))
  ((delta >= 2 && delta <= 90)) || fail "all-features rendered grade delta is invalid: $delta"
}

all_features_caption_frame_metrics() {
  ffmpeg -nostdin -v error -i "$1" -vf 'crop=260:24:110:202,format=rgb24' \
    -f rawvideo - | od -An -v -tu1 | awk -v pixels=6240 '
    { for (i=1; i<=NF; i++) {
        rgb[channel++]=$i
        if (channel==3) {
          if (rgb[0]<=75 && rgb[1]<=75 && rgb[2]<=75) dark++
          if (rgb[0]>=150 && rgb[1]>=150 && rgb[2]>=150) light++
          pixel++; channel=0
          if (pixel==pixels) {
            frames++; if (frames==1 || dark<min_dark) min_dark=dark
            if (frames==1 || light<min_light) min_light=light
            if (dark<3000 || light<35) bad++
            pixel=0; dark=0; light=0
          }
        }
      }
    }
    END { if (pixel || channel) exit 1; print frames+0,min_dark+0,min_light+0,bad+0 }'
}

all_features_assert_caption_readability() {
  local video=$1 frames dark light bad
  read -r frames dark light bad <<<"$(all_features_caption_frame_metrics "$video")"
  [[ $frames =~ ^[0-9]+$ && $frames -ge 93 && $dark -ge 3000 && $light -ge 35 && $bad == 0 ]] ||
    fail "all-features caption contrast is not continuous: frames=$frames dark=$dark light=$light bad=$bad"
}

all_features_assert_panel_and_caption() {
  local video=$1 before first last after edge caption time
  before=$(region_color_count "$video" 0.75 '360:45:60:205' navy)
  first=$(region_color_count "$video" 1.1 '360:45:60:205' navy)
  last=$(region_color_count "$video" 5.8 '360:45:60:205' navy)
  after=$(region_color_count "$video" 6.1 '360:45:60:205' navy)
  edge=$(region_color_count "$video" 3.5 '45:45:0:205' navy)
  ((first > before + 4000 && last > after + 4000 && edge < first / 3)) ||
    fail "all-features lower-third lifecycle failed: $before/$first/$last/$after edge=$edge"
  for time in 0.5 3.5 7.5; do
    caption=$(region_color_count "$video" "$time" '420:55:30:185' white)
    ((caption > 35)) || fail "all-features caption missing at ${time}s: $caption"
  done
  all_features_assert_caption_readability "$video"
}

check_all_features_visual_evidence() {
  local dir=${1:-"$PREVIEW_ROOT/all-features"}
  [[ -d $dir ]] || return 0
  local author preview plan video source uri
  author=$(example_authoring_canonical "$dir")
  preview=$(example_preview_canonical "$dir")
  plan=$(delivery_plan_path "$dir" master)
  video=$(delivery_video_path "$dir" master all-features.mp4)
  for file in "$author" "$preview" "$plan" "$video"; do require_file "$file"; done
  all_features_authoring_visual_contract "$author"
  all_features_preview_visual_contract "$preview"
  all_features_plan_visual_contract "$plan" "$author"
  uri=$(jq -er --arg shot "$(canonical_clip_id "$preview" shot)" '
    first(.project.sequences[].tracks[].clips[] | select(.id==$shot)).source.material_id as $id |
    first(.project.materials[] | select(.id==$id)).source.uri' "$preview")
  [[ $uri == assets/* && $uri != *..* ]] || fail "all-features unsafe footage URI: $uri"
  source="$dir/project/$uri"; require_file "$source"
  video_contract "$video" 7.9
  assert_stream_field "$video" v:0 width 480 "all-features video"
  assert_stream_field "$video" v:0 height 270 "all-features video"
  assert_stream_field "$video" v:0 codec_name h264 "all-features video"
  assert_stream_field "$video" v:0 pix_fmt yuv420p "all-features video"
  all_features_assert_mapping "$video" 0.5 "$source" 2.5 3.5 "all-features mapping at 0.5s"
  all_features_assert_mapping "$video" 3.5 "$source" 5.5 4.5 "all-features mapping at 3.5s"
  all_features_assert_mapping "$video" 7.5 "$source" 9.5 8.5 "all-features mapping at 7.5s"
  all_features_assert_grade "$video" "$source"
  all_features_assert_panel_and_caption "$video"
}

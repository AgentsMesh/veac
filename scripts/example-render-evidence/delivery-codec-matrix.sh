#!/usr/bin/env bash

codec_matrix_canonical_contract() {
  local canonical=$1 label=$2 width=$3 height=$4 fps=$5
  jq -e --argjson width "$width" --argjson height "$height" --argjson fps "$fps" '
    first(.project.render_configs[] | select(.id == "out_codec-matrix")) as $out |
    $out.sequence_id == "seq_main" and
    $out.raster == {"captions":"burn_in","frame_rate":{"denominator":1,"numerator":$fps},
      "height":$height,"width":$width} and
    ($out.deliverables | length) == 2 and
    first($out.deliverables[] | select(.id == "dlv_hevc-main10")) as $hevc |
    $hevc.target == {"name":"hevc.mov","type":"file"} and $hevc.kind.type == "video" and
    $hevc.kind.settings == {"audio":null,"container":"mov","hardware":{"type":"software"},
      "optimize_for_streaming":false,"pass_mode":"single","video":{"alpha":"opaque",
      "b_frames":null,"codec":"h265","color_space":{"matrix":"bt2020_ncl","primaries":"bt2020",
      "range":"limited","transfer":"smpte2084"},"gop_size":null,"level":null,
      "pixel_format":"yuv420p10le","profile":"h265_main10","rate_control":{"type":"crf","value":28}}} and
    first($out.deliverables[] | select(.id == "dlv_vp9-opus")) as $vp9 |
    $vp9.target == {"name":"vp9.webm","type":"file"} and $vp9.kind.type == "video" and
    $vp9.kind.settings == {"audio":{"channels":2,"codec":"opus","sample_rate":48000},
      "container":"webm","hardware":{"type":"software"},"optimize_for_streaming":false,
      "pass_mode":"single","video":{"alpha":"opaque","b_frames":null,"codec":"vp9",
      "color_space":null,"gop_size":null,"level":null,"pixel_format":"yuv420p",
      "profile":"vp9_profile0","rate_control":{"type":"crf","value":30}}}
  ' "$canonical" >/dev/null || fail "delivery-codec-matrix $label recipe contract failed"
}

codec_matrix_plan_contract() {
  local preview=$1 plan=$2
  jq -e --slurpfile preview "$preview" '
    first($preview[0].project.render_configs[] | select(.id == "out_codec-matrix")) as $config |
    .output.id == "pout_codec-matrix" and .output.render_config_id == $config.id and
    .output.sequence_id == $config.sequence_id and .output.raster == $config.raster and
    .output.deliverables == $config.deliverables and
    first(.sequences[] | select(.id == "seq_main")).duration == {"timescale":1000,"value":1000}
  ' "$plan" >/dev/null || fail "delivery-codec-matrix preview plan contract failed"
}

codec_matrix_format_field() {
  ffprobe -v error -show_entries "format=$2" -of default=nw=1:nk=1 "$1" | head -n 1
}

codec_matrix_assert_decode() {
  ffmpeg -nostdin -hide_banner -v error -xerror -i "$1" -map "$2" -f null - >/dev/null 2>&1 ||
    fail "$3 does not fully decode"
}

codec_matrix_assert_video_decode() {
  local file=$1 expected=$2 label=$3 count
  codec_matrix_assert_decode "$file" 0:v:0 "$label"
  count=$(ffprobe -v error -count_frames -select_streams v:0 \
    -show_entries stream=nb_read_frames -of default=nw=1:nk=1 "$file" 2>/dev/null)
  [[ $count == "$expected" ]] || fail "$label does not fully decode: expected $expected frames, got ${count:-none}"
}

codec_matrix_assert_hevc_levels() {
  local count
  count=$(ffmpeg -nostdin -v error -i "$1" -map 0:v:0 -frames:v 1 -pix_fmt gray16le \
    -f rawvideo - | od -An -v -tu2 | awk '{for(i=1;i<=NF;i++) seen[$i]=1} END {for(i in seen)n++; print n+0}')
  ((count > 256)) || fail "delivery-codec-matrix HEVC has only $count distinct 10-bit luma levels"
}

codec_matrix_expected_media() {
  jq -er '
    .output as $out | first(.sequences[] | select(.id == $out.sequence_id)).duration as $duration |
    [$out.raster.width,$out.raster.height,
     (($out.raster.frame_rate.numerator|tostring)+"/"+($out.raster.frame_rate.denominator|tostring)),
     ($duration.value/$duration.timescale)] | @tsv
  ' "$1"
}

codec_matrix_assert_hevc() {
  local file=$1 width=$2 height=$3 rate=$4 duration=$5 format brand frames
  format=$(codec_matrix_format_field "$file" format_name)
  brand=$(ffprobe -v error -show_entries format_tags=major_brand -of default=nw=1:nk=1 "$file")
  [[ $format == *mov* && $brand == qt* ]] || fail "delivery-codec-matrix HEVC is not a QuickTime MOV"
  assert_stream_count "$file" v 1 "delivery-codec-matrix HEVC"
  assert_stream_count "$file" a 0 "delivery-codec-matrix HEVC"
  assert_stream_field "$file" v:0 codec_name hevc "delivery-codec-matrix HEVC"
  assert_stream_field "$file" v:0 profile 'Main 10' "delivery-codec-matrix HEVC"
  assert_stream_field "$file" v:0 pix_fmt yuv420p10le "delivery-codec-matrix HEVC"
  assert_stream_field "$file" v:0 color_primaries bt2020 "delivery-codec-matrix HEVC"
  assert_stream_field "$file" v:0 color_transfer smpte2084 "delivery-codec-matrix HEVC"
  assert_stream_field "$file" v:0 color_space bt2020nc "delivery-codec-matrix HEVC"
  assert_stream_field "$file" v:0 color_range tv "delivery-codec-matrix HEVC"
  assert_stream_field "$file" v:0 width "$width" "delivery-codec-matrix HEVC"
  assert_stream_field "$file" v:0 height "$height" "delivery-codec-matrix HEVC"
  assert_stream_field "$file" v:0 r_frame_rate "$rate" "delivery-codec-matrix HEVC"
  assert_duration_close "$file" "$duration" 0.12 "delivery-codec-matrix HEVC"
  frames=$(awk -v rate="$rate" -v duration="$duration" 'BEGIN {split(rate,r,"/"); print int(duration*r[1]/r[2]+.5)}')
  codec_matrix_assert_video_decode "$file" "$frames" "delivery-codec-matrix HEVC"
  codec_matrix_assert_hevc_levels "$file"
}

codec_matrix_assert_vp9() {
  local file=$1 width=$2 height=$3 rate=$4 duration=$5 format frames
  format=$(codec_matrix_format_field "$file" format_name)
  [[ $format == *webm* ]] || fail "delivery-codec-matrix VP9 artifact is not WebM"
  assert_stream_count "$file" v 1 "delivery-codec-matrix VP9"
  assert_stream_count "$file" a 1 "delivery-codec-matrix VP9"
  assert_stream_field "$file" v:0 codec_name vp9 "delivery-codec-matrix VP9"
  assert_stream_field "$file" v:0 profile 'Profile 0' "delivery-codec-matrix VP9"
  assert_stream_field "$file" v:0 pix_fmt yuv420p "delivery-codec-matrix VP9"
  assert_stream_field "$file" v:0 width "$width" "delivery-codec-matrix VP9"
  assert_stream_field "$file" v:0 height "$height" "delivery-codec-matrix VP9"
  assert_stream_field "$file" v:0 r_frame_rate "$rate" "delivery-codec-matrix VP9"
  assert_stream_field "$file" a:0 codec_name opus "delivery-codec-matrix Opus"
  assert_stream_field "$file" a:0 sample_rate 48000 "delivery-codec-matrix Opus"
  assert_stream_field "$file" a:0 channels 2 "delivery-codec-matrix Opus"
  assert_duration_close "$file" "$duration" 0.12 "delivery-codec-matrix VP9"
  assert_non_silent_window "$file" 0 0.8 "delivery-codec-matrix Opus track"
  frames=$(awk -v rate="$rate" -v duration="$duration" 'BEGIN {split(rate,r,"/"); print int(duration*r[1]/r[2]+.5)}')
  codec_matrix_assert_video_decode "$file" "$frames" "delivery-codec-matrix VP9 video"
  codec_matrix_assert_decode "$file" 0:a:0 "delivery-codec-matrix Opus audio"
}

check_delivery_codec_matrix_evidence() {
  local dir=${1:-"$PREVIEW_ROOT/delivery-codec-matrix"}
  [[ -d $dir ]] || return 0
  local author preview plan hevc vp9 width height rate duration
  author=$(example_authoring_canonical "$dir"); preview=$(example_preview_canonical "$dir")
  plan=$(example_preview_plan "$dir" out_codec-matrix)
  hevc=$(delivery_video_path "$dir" codec-matrix hevc-main10)
  vp9=$(delivery_video_path "$dir" codec-matrix vp9-opus)
  for file in "$author" "$preview" "$plan" "$hevc" "$vp9"; do require_file "$file"; done
  codec_matrix_canonical_contract "$author" authoring 640 360 24
  codec_matrix_canonical_contract "$preview" preview 480 270 12
  codec_matrix_plan_contract "$preview" "$plan"
  read -r width height rate duration <<<"$(codec_matrix_expected_media "$plan")"
  codec_matrix_assert_hevc "$hevc" "$width" "$height" "$rate" "$duration"
  codec_matrix_assert_vp9 "$vp9" "$width" "$height" "$rate" "$duration"
}

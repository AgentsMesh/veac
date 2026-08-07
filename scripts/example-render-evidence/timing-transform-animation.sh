#!/usr/bin/env bash
# shellcheck disable=SC2016

timing_red_sample() {
  local video=$1 time=$2
  ffmpeg -nostdin -hide_banner -loglevel error -i "$video" -frames:v 1 \
    -vf "trim=start=$time,setpts=PTS-STARTPTS,scale=240:135:flags=area,format=rgb24" \
    -f rawvideo - 2>/dev/null | od -An -v -tu1 | awk -v width=240 '
      { for (i=1;i<=NF;i++) { channel=bytes%3
          if (channel==0) r=$i; else if (channel==1) g=$i; else {
            pixel=int(bytes/3); x=pixel%width; y=int(pixel/width)
            if (r>135 && r>g+45 && r>$i+35) {
              sx+=x; sy+=y; count++; if (!found||x<minx) minx=x
              if (!found||x>maxx) maxx=x; if (!found||y<miny) miny=y
              if (!found||y>maxy) maxy=y; found=1 }
          } bytes++ } }
      END { if (!count) exit 1
        printf "%.2f %.2f %d %d %d %d %d\n",sx/count,sy/count,minx,maxx,miny,maxy,count }'
}

transform_sample_rows() {
  local video=$1 time sample
  shift
  for time in "$@"; do
    sample=$(timing_red_sample "$video" "$time") || return 1
    printf '%s %s\n' "$time" "$sample"
  done
}

assert_segmented_transform_motion() {
  local video=$1 samples
  samples=$(transform_sample_rows "$video" 0.25 0.4 0.6 0.95 1.4 1.85 2.3 2.65) ||
    fail 'transform badge is not visible throughout the segmented motion'
  awk '
    function abs(v) { return v<0 ? -v : v }
    NR==1 { held=$2; previous=$2; next }
    NR==2 { if (abs($2-held)>1.5) bad=1; previous=$2; next }
    { if (!($2<previous-1.0)) bad=1; previous=$2 }
    END { exit !(NR==8 && !bad) }
  ' <<<"$samples" || fail "transform hold/segment trajectory is wrong: $samples"
}

assert_spring_transform_motion() {
  local video=$1 samples
  samples=$(transform_sample_rows "$video" 2.75 3.0 3.3 3.6 3.78) ||
    fail 'transform badge is not visible throughout the Spring motion'
  awk '
    NR==1 { first=$2; next }
    NR==2 { second=$2; late_min=$2; late_max=$2; next }
    { if ($2<late_min) late_min=$2; if ($2>late_max) late_max=$2 }
    END { exit !(NR==5 && second>first+8 && late_max-late_min<=3) }
  ' <<<"$samples" || fail "transform Spring trajectory does not advance and settle: $samples"
}

assert_authored_opacity_visible() {
  local video=$1 early full late
  early=$(timing_red_sample "$video" 0.08 2>/dev/null | awk '{print $7+0}') || early=0
  full=$(timing_red_sample "$video" 0.3 | awk '{print $7+0}') || full=0
  late=$(timing_red_sample "$video" 3.92 2>/dev/null | awk '{print $7+0}') || late=0
  ((full > early * 2 && full > late * 2)) ||
    fail "authored opacity fade is not visible: early=$early full=$full late=$late"
}

assert_authored_temporal_contract() {
  local canonical=$1 badge=$2
  jq -e --arg badge "$badge" '
    first(.project.sequences[].tracks[].clips[] | select(.id==$badge)) as $clip |
    $clip.visual.opacity.type == "binding" and
    $clip.visual.opacity.binding_id as $binding_id |
    first(.temporal.bindings[] | select(.id==$binding_id)) as $binding |
    $binding.result_type == "scalar" and ($binding.clocks | length) == 1 and
    $binding.clocks[0].clock == "progress" and
    $binding.clocks[0].owner == {"type":"item","item_id":$badge} and
    any(.temporal.programs[]; .id==$binding.program_id) and
    first(.temporal.provenance[] | select(.id==$binding.provenance_id)) as $origin |
    $origin.definition.name == "visual-opacity" and $origin.origin.function == "animate"
  ' "$canonical" >/dev/null || fail 'authored temporal opacity contract failed'
}

assert_transform_plan() {
  local plan=$1 badge=$2
  jq -e --arg badge "$badge" '
    def badge: first(.sequences[].tracks[].clips[] | select(.id==$badge));
    badge as $clip | $clip.visual.transform.position as $position |
    $position.type == "keyframes" and ($position.keyframes | length) == 8 and
    ($position.keyframes | map(.time)) == [
      {"timescale":600,"value":0},{"timescale":600,"value":270},
      {"timescale":600,"value":540},{"timescale":600,"value":810},
      {"timescale":600,"value":1080},{"timescale":600,"value":1350},
      {"timescale":600,"value":1620},{"timescale":600,"value":2400}] and
    ($position.keyframes | map(.interpolation.type)) ==
      ["hold","linear","ease_in","ease_out","ease_in_out","cubic_bezier","spring","linear"] and
    $clip.visual.opacity.type == "binding" and
    any(.temporal.bindings[]; .id==$clip.visual.opacity.binding_id and
      .clocks[0].clock=="progress") and
    $clip.visual.transform.crop.value == {"x":0.14,"y":0.04,"width":0.84,"height":0.92} and
    $clip.visual.transform.scale.value == {"x":1.12,"y":0.92} and
    $clip.visual.transform.shear == {"x":0.22,"y":-0.08} and
    $clip.visual.transform.rotation_degrees.value == 5 and
    $clip.visual.transform.anchor == {"x":1,"y":1} and
    $clip.visual.placement == {"anchor":"bottom_right","inset":{"x":80,"y":80},"type":"anchor"} and
    $clip.visual.transform.flip_horizontal and ($clip.visual.transform.flip_vertical | not)
  ' "$plan" >/dev/null || fail 'transform keyframe and authored temporal plan contract failed'
}

check_transform_animation_evidence() {
  local dir=$1 author plan video badge
  author=$(example_authoring_canonical "$dir"); plan=$(timing_plan "$dir")
  video=$(timing_video "$dir"); badge=$(canonical_clip_id "$author" badge)
  assert_authored_temporal_contract "$author" "$badge"
  assert_transform_plan "$plan" "$badge"
  video_contract "$video" 3.9
  assert_authored_opacity_visible "$video"
  assert_segmented_transform_motion "$video"
  assert_spring_transform_motion "$video"
}

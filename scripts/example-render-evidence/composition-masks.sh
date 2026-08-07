#!/usr/bin/env bash

masks_canonical_contract() {
  local canonical=$1 label=$2 root=${3:-.project} identity=${4:-$1}
  local circle rectangle ellipse rounded polygon path combined
  circle=$(canonical_clip_id "$identity" circle)
  rectangle=$(canonical_clip_id "$identity" rectangle)
  ellipse=$(canonical_clip_id "$identity" ellipse)
  rounded=$(canonical_clip_id "$identity" rounded)
  polygon=$(canonical_clip_id "$identity" polygon)
  path=$(canonical_clip_id "$identity" path)
  combined=$(canonical_clip_id "$identity" combined)
  jq -e --arg root "$root" --arg circle "$circle" --arg rectangle "$rectangle" \
    --arg ellipse "$ellipse" --arg rounded "$rounded" --arg polygon "$polygon" \
    --arg path "$path" --arg combined "$combined" '
    def clips: if $root == ".project" then .project.sequences[].tracks[].clips[]
      else .sequences[].tracks[].clips[] end;
    def clip($id): first(clips | select(.id == $id));
    def seconds: .value/.timescale;
    def centered($id): clip($id) as $clip |
      $clip.visual.placement == {"anchor":"center","inset":{"x":0,"y":0},"type":"anchor"} and
      $clip.visual.frame == null and $clip.visual.transform.anchor == {"x":0.5,"y":0.5};
    def sample($id;$start;$shape): clip($id) as $clip |
      centered($id) and ($clip.record_range.start|seconds)==$start and
      ($clip.record_range.duration|seconds)==1 and ($clip.visual.masks|length)==1 and
      $clip.visual.masks[0].shape.type==$shape and
      $clip.visual.masks[0].position.value=={"x":0.5,"y":0.5};
    sample($circle;0;"circle") and sample($rectangle;1;"rectangle") and
    sample($ellipse;2;"ellipse") and sample($rounded;3;"rounded_rectangle") and
    sample($polygon;4;"polygon") and sample($path;5;"path") and
    centered($combined) and (clip($combined).record_range.start|seconds)==6 and
    (clip($combined).record_range.duration|seconds)==2 and
    (clip($combined).visual.masks|map(.shape.type))==
      ["circle","rectangle","ellipse","rounded_rectangle","polygon","path"]
  ' "$canonical" >/dev/null || fail "masks-and-mattes $label centered mask contract failed"
}

mask_color_footprint() {
  local video=$1 time=$2
  ffmpeg -v error -ss "$time" -i "$video" -frames:v 1 -vf format=rgb24 -f rawvideo - |
    od -An -v -tu1 | awk -v width=480 '
      { for (i=1;i<=NF;i++) { rgb[channel++]=$i
          if (channel==3) { max=rgb[0]; min=rgb[0]
            if (rgb[1]>max) max=rgb[1]; if (rgb[2]>max) max=rgb[2]
            if (rgb[1]<min) min=rgb[1]; if (rgb[2]<min) min=rgb[2]
            hit=(max-min>42 && max>105); x=pixel%width; y=int(pixel/width)
            if (hit) { count++; sumx+=x; sumy+=y
              if (!seen||x<minx) minx=x; if (!seen||x>maxx) maxx=x
              if (!seen||y<miny) miny=y; if (!seen||y>maxy) maxy=y
              if (x<2||x>477||y<2||y>267) edge++; seen=1 }
            if (x>=235&&x<=245&&y>=130&&y<=140) center+=hit
            channel=0; pixel++ } } }
      END { print count+0,minx+0,maxx+0,miny+0,maxy+0,
        count?sumx/count:0,count?sumy/count:0,edge+0,center+0 }'
}

assert_centered_mask_footprint() {
  local video=$1 time=$2 values=() width height
  read -r -a values <<<"$(mask_color_footprint "$video" "$time")"
  width=$((values[2]-values[1]+1)); height=$((values[4]-values[3]+1))
  awk -v count="${values[0]}" -v width="$width" -v height="$height" \
    -v cx="${values[5]}" -v cy="${values[6]}" -v edge="${values[7]}" '
      BEGIN { exit !(count>7000 && width>120 && height>80 && width<470 && height<265 &&
        cx>205 && cx<275 && cy>105 && cy<165 && edge==0) }' ||
    fail "mask stage at ${time}s is clipped, off-center, or label-only: ${values[*]}"
}

assert_inverted_mask_footprint() {
  local video=$1 time=$2 values=()
  read -r -a values <<<"$(mask_color_footprint "$video" "$time")"
  ((values[0] > 18000 && values[1] <= 2 && values[2] >= 477 &&
    values[3] <= 2 && values[4] >= 267 && values[7] > 500 && values[8] == 0)) ||
    fail "inverted alpha stage at ${time}s lacks centered outer footprint: ${values[*]}"
}

#!/usr/bin/env bash

stabilization_canonical_contract() {
  local canonical=$1 label=$2 root=${3:-.project} identity=${4:-$1} before after effect
  before=$(canonical_clip_id "$identity" stabilize-before)
  after=$(canonical_clip_id "$identity" stabilize-after)
  effect=$(jq -er --arg after "$after" '
    [.project.sequences[].tracks[].clips[] | select(.id == $after) | .effects[].id] |
    if length == 1 then .[0] else error("stabilization effect is not unique") end
  ' "$identity") || fail "cannot resolve stabilization effect"
  jq -e --arg root "$root" --arg before "$before" --arg after "$after" \
    --arg effect "$effect" '
    def seconds: .value / .timescale;
    def clips: if $root == ".project" then .project.sequences[].tracks[].clips[]
      else .sequences[].tracks[].clips[] end;
    def clip($id): first(clips | select(.id == $id));
    clip($before) as $before | clip($after) as $after |
    ($before.record_range.start | seconds) == 4 and
    ($before.record_range.duration | seconds) == 2 and
    ($after.record_range.start | seconds) == 6 and
    ($after.record_range.duration | seconds) == 2 and
    (if $root == ".project" then
      $before.source.type == "media" and $after.source == $before.source and
      $after.source_mapping == $before.source_mapping and
      ($before.source_mapping.time_map.source_start | seconds) == 0
    else
      $before.source.type == "media" and $before.source.audio_stream == null and
      $before.source.video_stream == {"global_index":0,"type_index":0} and
      $after.source == $before.source and $after.source_mapping == $before.source_mapping and
      ($before.source_mapping.time_map.source_range_per_repeat.start | seconds) == 0 and
      ($before.source_mapping.time_map.source_range_per_repeat.duration | seconds) == 2
    end) and
    ($before.effects | length) == 0 and ($after.effects | length) == 1 and
    (if $root == ".project" then
      $after.effects[0] == {"effect":{"type":"video_stabilize","enabled":true},
        "enable_range":null,"enabled":true,"id":$effect}
    else
      $after.effects[0].id == $effect and
      $after.effects[0].effect == {"type":"video_stabilize","enabled":true} and
      ($after.effects[0].active_range.start | seconds) == 0 and
      ($after.effects[0].active_range.duration | seconds) == 2
    end)
  ' "$canonical" >/dev/null || fail "video-effects $label stabilization contract failed"
}

stabilization_grid_phase() {
  local video=$1 time=$2
  ffmpeg -v error -ss "$time" -i "$video" -frames:v 1 -vf format=rgb24 -f rawvideo - |
    od -An -v -tu1 | awk -v width=480 -v period=36 '
      { for (i=1;i<=NF;i++) { rgb[channel++]=$i
          if (channel==3) { x=pixel%width; y=int(pixel/width)
            if (rgb[0]>205 && rgb[1]>205 && rgb[2]>205) { hx[x%period]++; hy[y%period]++; total++ }
            channel=0; pixel++ } } }
      END {
        for (i=0;i<period;i++) { sx+=hx[i]; sy+=hy[i]
          if (hx[i]>xpeak) { xpeak=hx[i]; xp=i } if (hy[i]>ypeak) { ypeak=hy[i]; yp=i } }
        if (!total) exit 2
        printf "%d %d %d %.2f %d %.2f\n",xp,yp,xpeak,sx/period,ypeak,sy/period
      }'
}

stabilization_motion_score() {
  local video=$1; shift
  local phases=() time xp yp xpeak xavg ypeak yavg
  for time in "$@"; do
    read -r xp yp xpeak xavg ypeak yavg <<<"$(stabilization_grid_phase "$video" "$time")"
    [[ -n ${yavg:-} ]] || fail "video stabilization grid landmark is missing at ${time}s"
    awk -v peak="$xpeak" -v avg="$xavg" 'BEGIN { exit !(peak >= avg*2.2) }' ||
      fail "video stabilization vertical grid landmark is missing at ${time}s"
    awk -v peak="$ypeak" -v avg="$yavg" 'BEGIN { exit !(peak >= avg*2.2) }' ||
      fail "video stabilization horizontal grid landmark is missing at ${time}s"
    phases+=("$xp:$yp")
  done
  printf '%s\n' "${phases[@]}" | awk -F: -v period=36 '
    NR==1 { px=$1; py=$2; next }
    { dx=$1-px; dy=$2-py; if (dx>period/2) dx-=period; if (dx < -period/2) dx+=period
      if (dy>period/2) dy-=period; if (dy < -period/2) dy+=period
      total+=(dx<0?-dx:dx)+(dy<0?-dy:dy); count++; px=$1; py=$2 }
    END { if (count) printf "%.3f\n",total/count; else exit 1 }'
}

stabilization_unique_count() {
  local video=$1; shift
  local time phases=() xp yp rest
  for time in "$@"; do
    read -r xp yp rest <<<"$(stabilization_grid_phase "$video" "$time")"
    phases+=("$xp:$yp")
  done
  printf '%s\n' "${phases[@]}" | sort -u | wc -l | tr -d ' '
}

stabilization_content_signature() {
  local video=$1 time=$2 xp=$3 yp=$4 reference_x=$5 reference_y=$6
  local dx=$((xp-reference_x)) dy=$((yp-reference_y)) x y
  ((dx > 18)) && dx=$((dx-36)); ((dx < -18)) && dx=$((dx+36))
  ((dy > 18)) && dy=$((dy-36)); ((dy < -18)) && dy=$((dy+36))
  x=$((18+dx)); y=$((18+dy))
  ffmpeg -v error -ss "$time" -i "$video" -frames:v 1 \
    -vf "crop=444:234:${x}:${y},scale=8:6:flags=area,format=gray,lutyuv=y='floor(val/32)*32'" \
    -f framemd5 - | awk '/^[^#]/{print $NF}'
}

stabilization_content_unique_count() {
  local video=$1; shift
  local time xp yp rest reference_x='' reference_y='' signatures=()
  for time in "$@"; do
    read -r xp yp rest <<<"$(stabilization_grid_phase "$video" "$time")"
    if [[ -z $reference_x ]]; then reference_x=$xp; reference_y=$yp; fi
    signatures+=("$(stabilization_content_signature \
      "$video" "$time" "$xp" "$yp" "$reference_x" "$reference_y")")
  done
  printf '%s\n' "${signatures[@]}" | sort -u | wc -l | tr -d ' '
}

stabilization_assert_rendered_motion() {
  local video=$1 before after unique content
  local samples=(0.1 0.3 0.5 0.7 0.9 1.1 1.3 1.5 1.7 1.9) before_times=() after_times=() value
  for value in "${samples[@]}"; do
    before_times+=("$(awk -v v="$value" 'BEGIN { print 4+v }')")
    after_times+=("$(awk -v v="$value" 'BEGIN { print 6+v }')")
  done
  before=$(stabilization_motion_score "$video" "${before_times[@]}")
  after=$(stabilization_motion_score "$video" "${after_times[@]}")
  awk -v value="$before" 'BEGIN { exit !(value >= 5) }' ||
    fail "video stabilization before segment is already stable: motion $before"
  awk -v before="$before" -v after="$after" 'BEGIN { exit !(after <= 4 && after <= before*.45) }' ||
    fail "video stabilization does not reduce grid motion: before=$before after=$after"
  unique=$(stabilization_unique_count "$video" "${after_times[@]}")
  ((unique >= 4)) || fail "video stabilization after segment is frozen: $unique distinct frames"
  content=$(stabilization_content_unique_count "$video" "${after_times[@]}")
  ((content >= 4)) || fail "video stabilization after segment content is frozen: $content signatures"
}

check_video_stabilization_evidence() {
  local dir=${1:-"$PREVIEW_ROOT/video-effects"}
  [[ -d $dir ]] || return 0
  local author preview plan video
  author=$(example_authoring_canonical "$dir"); preview=$(example_preview_canonical "$dir")
  plan=$(delivery_plan_path "$dir" preview); video=$(delivery_video_path "$dir" preview preview.mp4)
  for file in "$author" "$preview" "$plan" "$video"; do require_file "$file"; done
  assert_resolution_chain "$dir" "$plan" "$video" video-effects
  stabilization_canonical_contract "$author" authoring
  stabilization_canonical_contract "$preview" preview .project "$author"
  stabilization_canonical_contract "$plan" plan .plan "$author"
  video_contract "$video" 7.9
  stabilization_assert_rendered_motion "$video"
}

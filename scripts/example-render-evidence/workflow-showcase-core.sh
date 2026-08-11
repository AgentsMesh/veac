#!/usr/bin/env bash

workflow_material_id() {
  local canonical=$1 logical_key=$2
  jq -er --arg key "$logical_key" '
    [.project.materials[] |
      select(.authorship.logical_path[-1] == $key) | .id] |
    if length == 1 then .[0] else error("material logical key is not unique") end
  ' "$canonical" || fail "cannot resolve material logical key: $logical_key"
}

workflow_effect_id() {
  local canonical=$1 clip_id=$2 effect_type=$3
  jq -er --arg clip "$clip_id" --arg type "$effect_type" '
    [.project.sequences[].tracks[].clips[] | select(.id == $clip) |
      .effects[] | select(.effect.type == $type) | .id] |
    if length == 1 then .[0] else error("effect identity is not unique") end
  ' "$canonical" || fail "cannot resolve $effect_type on $clip_id"
}

workflow_track_id_for_clip() {
  local canonical=$1 clip_id=$2
  jq -er --arg clip "$clip_id" '
    [.project.sequences[].tracks[] | select(any(.clips[]; .id == $clip)) | .id] |
    if length == 1 then .[0] else error("clip owner track is not unique") end
  ' "$canonical" || fail "cannot resolve owner track for $clip_id"
}

workflow_plan_base_contract() {
  local plan=$1 identity=$2 delivery=$3 target=$4 duration=$5 audio=$6 label=$7
  local policy=${8:-veac.default-stream.v1} sequence
  sequence=$(canonical_sequence_id "$identity" main)
  jq -e --arg sequence "$sequence" --arg delivery "$delivery" --arg target "$target" \
    --arg policy "$policy" --argjson duration "$duration" --argjson audio "$audio" '
    def seconds: .value / .timescale;
    .header.schema == "https://veac.dev/schemas/render-plan" and
    .header.schema_version == 6 and
    .header.resolver == {capability_profile:"backend-neutral-v1",
      effect_registry_version:"veac-ir-effects-v3",
      resolver_version:"veac-plan-resolver-v6",
      stream_selection_policy:$policy} and
    .header.source.timebase == 600 and .entry_sequence_id == $sequence and
    .output.sequence_id == $sequence and
    .output.raster == {captions:"discard",frame_rate:{denominator:1,numerator:12},
      height:270,width:480} and
    (.output.deliverables | length) == 1 and
    .output.deliverables[0].target == {name:$target,type:"file"} and
    .output.deliverables[0].kind == {type:"video",settings:{
      audio:(if $audio then {channels:2,codec:"aac",sample_rate:48000} else null end),
      container:"mp4",hardware:{type:"software"},optimize_for_streaming:true,
      pass_mode:"single",video:{alpha:"opaque",b_frames:null,codec:"h264",
        color_space:null,gop_size:null,level:null,pixel_format:"yuv420p",profile:"h264_high",
        rate_control:{type:"crf",value:20}}}} and
    ([.sequences[] | select(.id == $sequence)] | length) == 1 and
    first(.sequences[] | select(.id == $sequence)) as $sequence_value |
    ($sequence_value.duration | seconds) == $duration and
    $sequence_value.settings == {frame_rate:{denominator:1,numerator:12},height:360,
      sample_rate:48000,width:640} and .temporal.opset_version == 1
  ' "$plan" >/dev/null || fail "$label render-plan envelope contract failed"
  [[ $delivery =~ ^[a-z][a-z0-9-]*$ ]] || fail "$label has unsafe delivery key"
}

workflow_video_contract() {
  local video=$1 duration=$2 label=$3
  video_contract "$video" "$(awk -v value="$duration" 'BEGIN { print value-.1 }')"
  assert_duration_close "$video" "$duration" 0.08 "$label"
  assert_stream_field "$video" v:0 width 480 "$label"
  assert_stream_field "$video" v:0 height 270 "$label"
}

workflow_assert_region_rgb() {
  local video=$1 time=$2 crop=$3 tolerance=$7 label=$8 sampled
  local expected=("$4" "$5" "$6") actual=()
  sampled=$(region_rgb "$video" "$time" "$crop")
  read -r -a actual <<< "$sampled"
  ((${#actual[@]} == 3)) || fail "$label: RGB sample is unavailable"
  local index delta
  for index in 0 1 2; do
    delta=$((actual[index] - expected[index])); ((delta < 0)) && delta=$((-delta))
    ((delta <= tolerance)) || fail "$label: RGB $sampled is outside tolerance"
  done
}

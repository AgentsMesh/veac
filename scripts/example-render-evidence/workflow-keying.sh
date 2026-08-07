#!/usr/bin/env bash

workflow_keying_contract() {
  local doc=$1 mode=$2 identity=$3 label=$4
  local chroma luma stable chroma_fx spill_fx luma_fx stabilize_fx plate shaky
  chroma=$(canonical_clip_id "$identity" chroma); luma=$(canonical_clip_id "$identity" luma)
  stable=$(canonical_clip_id "$identity" stable)
  plate=$(workflow_material_id "$identity" plate); shaky=$(workflow_material_id "$identity" shaky)
  chroma_fx=$(workflow_effect_id "$identity" "$chroma" video_chroma_key)
  spill_fx=$(workflow_effect_id "$identity" "$chroma" video_chroma_spill)
  luma_fx=$(workflow_effect_id "$identity" "$luma" video_luma_key)
  stabilize_fx=$(workflow_effect_id "$identity" "$stable" video_stabilize)
  jq -e --arg mode "$mode" --arg chroma "$chroma" --arg luma "$luma" \
    --arg stable "$stable" --arg plate "$plate" --arg shaky "$shaky" \
    --arg chroma_fx "$chroma_fx" --arg spill_fx "$spill_fx" \
    --arg luma_fx "$luma_fx" --arg stabilize_fx "$stabilize_fx" '
    def seconds: .value / .timescale;
    def clips: if $mode == "plan" then .sequences[].tracks[].clips[]
      else .project.sequences[].tracks[].clips[] end;
    def one($id): [clips | select(.id == $id)] |
      if length == 1 then .[0] else error("clip identity is not unique") end;
    def range($clip;$start;$duration):
      ($clip.record_range.start | seconds) == $start and
      ($clip.record_range.duration | seconds) == $duration;
    def authored_map: {frame_synthesis:"nearest",out_of_range:"strict",time_map:{
      direction:"forward",rate:{denominator:1,numerator:1},repeat:1,
      source_start:{timescale:600,value:0},type:"linear"}};
    def resolved_map($duration): {frame_synthesis:"nearest",out_of_range:"strict",time_map:{
      direction:"forward",rate:{denominator:1,numerator:1},repeat:1,
      source_range_per_repeat:{start:{timescale:600,value:0},
        duration:{timescale:600,value:$duration}},type:"linear"}};
    one($chroma) as $chroma | one($luma) as $luma | one($stable) as $stable |
    range($chroma;0;1) and range($luma;1;1) and range($stable;2;2) and
    (if $mode == "plan" then
      $chroma.source.type == "media" and $luma.source == $chroma.source and
      $stable.source.type == "media" and $stable.source.input_id != $chroma.source.input_id and
      $chroma.source.video_stream == {global_index:0,type_index:0} and
      $stable.source.video_stream == {global_index:0,type_index:0} and
      $chroma.source_mapping == resolved_map(600) and
      $luma.source_mapping == resolved_map(600) and
      $stable.source_mapping == resolved_map(1200)
    else
      $chroma.source == {material_id:$plate,type:"media"} and
      $luma.source == $chroma.source and $stable.source == {material_id:$shaky,type:"media"} and
      $chroma.source_mapping == authored_map and $luma.source_mapping == authored_map and
      $stable.source_mapping == authored_map
    end) and
    ($chroma.effects | map(.id)) == [$chroma_fx,$spill_fx] and
    $chroma.effects[0].effect == {blend:{type:"constant",value:0.08},
      color:{alpha:255,blue:100,green:194,red:45},
      similarity:{type:"constant",value:0.18},type:"video_chroma_key"} and
    $chroma.effects[1].effect == {amount:{type:"constant",value:1},
      color:{alpha:255,blue:100,green:194,red:45},
      range:{type:"constant",value:0.2},type:"video_chroma_spill"} and
    $luma.effects == [(if $mode == "plan" then
      {active_range:{start:{value:0,timescale:600},duration:{value:600,timescale:600}},
       effect:{invert:false,softness:{type:"constant",value:0.08},
         threshold:{type:"constant",value:0.12},tolerance:{type:"constant",value:0.08},
         type:"video_luma_key"},id:$luma_fx}
      else {effect:{invert:false,softness:{type:"constant",value:0.08},
        threshold:{type:"constant",value:0.12},tolerance:{type:"constant",value:0.08},
        type:"video_luma_key"},enable_range:null,enabled:true,id:$luma_fx} end)] and
    ($stable.effects | length) == 1 and $stable.effects[0].id == $stabilize_fx and
    $stable.effects[0].effect == {enabled:true,type:"video_stabilize"} and
    (if $mode == "plan" then
      all($chroma.effects[]; (.active_range.start | seconds) == 0 and
        (.active_range.duration | seconds) == 1) and
      ($stable.effects[0].active_range.start | seconds) == 0 and
      ($stable.effects[0].active_range.duration | seconds) == 2
    else ([$chroma.effects[],$stable.effects[]] |
      all(.enabled and .enable_range == null)) end)
  ' "$doc" >/dev/null || fail "keying-stabilization $label typed contract failed"
}

workflow_keying_labels_contract() {
  local doc=$1 mode=$2 identity=$3 label=$4 key stable
  key=$(canonical_clip_id "$identity" key-label); stable=$(canonical_clip_id "$identity" stable-label)
  jq -e --arg mode "$mode" --arg key "$key" --arg stable "$stable" '
    def clips: if $mode == "plan" then .sequences[].tracks[].clips[]
      else .project.sequences[].tracks[].clips[] end;
    def one($id): first(clips | select(.id == $id));
    def text($clip): if $mode == "plan" then $clip.source.content.text else $clip.source.text end;
    one($key) as $key | one($stable) as $stable |
    text($key) == "色度键、溢色抑制与亮度键" and
    text($stable) == "确定性抖动素材：防抖后" and
    $key.record_range == {start:{value:0,timescale:600},duration:{value:1200,timescale:600}} and
    $stable.record_range == {start:{value:1200,timescale:600},duration:{value:1200,timescale:600}}
  ' "$doc" >/dev/null || fail "keying-stabilization $label Chinese labels contract failed"
}

workflow_keying_input_contract() {
  local plan=$1 identity=$2 plate shaky chroma luma stable
  plate=$(workflow_material_id "$identity" plate); shaky=$(workflow_material_id "$identity" shaky)
  chroma=$(canonical_clip_id "$identity" chroma); luma=$(canonical_clip_id "$identity" luma)
  stable=$(canonical_clip_id "$identity" stable)
  jq -e --arg plate "$plate" --arg shaky "$shaky" --arg chroma "$chroma" \
    --arg luma "$luma" --arg stable "$stable" '
    def one($id): [.inputs[] | select(.material_id == $id)] |
      if length == 1 then .[0] else error("resolved input is not unique") end;
    def clip($id): first(.sequences[].tracks[].clips[] | select(.id == $id));
    one($plate) as $plate | one($shaky) as $shaky |
    $plate.canonical_uri == "assets/effects-plate.png" and
    $plate.kind == {material_kind:"image",type:"media"} and $plate.audio == null and
    $plate.video.selection == {global_index:0,type_index:0} and
    $shaky.canonical_uri == "assets/shaky.mp4" and
    $shaky.kind == {material_kind:"video",type:"media"} and $shaky.audio == null and
    $shaky.video.selection == {global_index:0,type_index:0} and
    ($shaky.video.duration.value / $shaky.video.duration.timescale) == 2 and
    clip($chroma).source.input_id == $plate.id and clip($luma).source.input_id == $plate.id and
    clip($stable).source.input_id == $shaky.id
  ' "$plan" >/dev/null || fail "keying-stabilization resolved input contract failed"
}

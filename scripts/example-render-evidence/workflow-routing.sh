#!/usr/bin/env bash

workflow_routing_contract() {
  local doc=$1 mode=$2 identity=$3 label=$4 music key tone music_track key_track relation sequence
  music=$(canonical_clip_id "$identity" music); key=$(canonical_clip_id "$identity" key)
  tone=$(workflow_material_id "$identity" tone)
  music_track=$(workflow_track_id_for_clip "$identity" "$music")
  key_track=$(workflow_track_id_for_clip "$identity" "$key")
  sequence=$(canonical_sequence_id "$identity" main)
  relation=$(jq -er --arg music "$music" '
    [.project.relations[] | select(.kind.type == "sidechain" and
      .kind.target.item_id == $music) | .id] |
    if length == 1 then .[0] else error("sidechain relation is not unique") end
  ' "$identity") || fail "cannot resolve audio-routing-ducking sidechain relation"
  jq -e --arg mode "$mode" --arg music "$music" --arg key "$key" \
    --arg tone "$tone" --arg music_track "$music_track" --arg key_track "$key_track" \
    --arg relation "$relation" --arg sequence "$sequence" '
    def seconds: .value / .timescale;
    def clips: if $mode == "plan" then .sequences[].tracks[].clips[]
      else .project.sequences[].tracks[].clips[] end;
    def one($id): [clips | select(.id == $id)] |
      if length == 1 then .[0] else error("clip identity is not unique") end;
    def owner($id): if $mode == "plan" then
      first(.sequences[].tracks[] | select(any(.clips[]; .id == $id)))
      else first(.project.sequences[].tracks[] | select(any(.clips[]; .id == $id))) end;
    def base_audio($gain;$pan): {crossfade:{curve:"equal_power",
      fade_in:{timescale:600,value:90},fade_out:{timescale:600,value:90}},
      gain:{type:"constant",value:$gain},muted:false,normalize:false,
      pan:{type:"constant",value:$pan},pitch_policy:"preserve",processors:[]};
    def authored_map: {frame_synthesis:"nearest",out_of_range:"strict",time_map:{
      direction:"forward",rate:{denominator:1,numerator:1},repeat:1,
      source_start:{timescale:600,value:0},type:"linear"}};
    def resolved_map: {frame_synthesis:"nearest",out_of_range:"strict",time_map:{
      direction:"forward",rate:{denominator:1,numerator:1},repeat:1,
      source_range_per_repeat:{start:{timescale:600,value:0},
        duration:{timescale:600,value:1200}},type:"linear"}};
    one($music) as $music_item | one($key) as $key_item |
    ($music_item.record_range.start | seconds) == 0 and
    ($music_item.record_range.duration | seconds) == 2 and
    $key_item.record_range == $music_item.record_range and
    (if $mode == "plan" then
      $music_item.source.type == "media" and $music_item.source == $key_item.source and
      $music_item.source.audio_stream == {global_index:0,type_index:0} and
      $music_item.source.video_stream == null and
      $music_item.source_mapping == resolved_map and $key_item.source_mapping == resolved_map and
      $music_item.audio == (base_audio(0.75;-0.25) + {sidechain:{active_range:null,
        attack_ms:10,ratio:4,relation_id:$relation,release_ms:120,
        source:{track_id:$key_track,type:"track"},threshold_db:-24}}) and
      $key_item.audio == (base_audio(0.5;0.25) + {sidechain:null}) and
      owner($music).routing == {audio:{bus_id:"bus_music-bus",type:"bus"},visual:null} and
      owner($key).routing == {audio:{type:"main_mix"},visual:null}
    else
      $music_item.source == {material_id:$tone,type:"media"} and
      $key_item.source == $music_item.source and
      $music_item.source_mapping == authored_map and $key_item.source_mapping == authored_map and
      $music_item.audio == base_audio(0.75;-0.25) and
      $key_item.audio == base_audio(0.5;0.25) and
      owner($music).id == $music_track and owner($key).id == $key_track and
      owner($music).routing == {bus_id:"bus_music-bus",type:"audio_bus"} and
      owner($key).routing == {type:"default"} and
      ([.project.relations[] | select(.id == $relation)] | length) == 1 and
      first(.project.relations[] | select(.id == $relation)).sequence_id == $sequence and
      first(.project.relations[] | select(.id == $relation)).kind == {
        key:{track_id:$key_track,type:"track"},parameters:{active_range:null,
          attack_ms:10,ratio:4,release_ms:120,threshold_db:-24},
        target:{item_id:$music,type:"item"},type:"sidechain"}
    end)
  ' "$doc" >/dev/null || fail "audio-routing-ducking $label typed contract failed"
}

workflow_routing_label_contract() {
  local doc=$1 mode=$2 identity=$3 label=$4 clip
  clip=$(canonical_clip_id "$identity" explanation)
  jq -e --arg mode "$mode" --arg clip "$clip" '
    def clips: if $mode == "plan" then .sequences[].tracks[].clips[]
      else .project.sequences[].tracks[].clips[] end;
    first(clips | select(.id == $clip)) as $item |
    $item.record_range == {start:{value:0,timescale:600},duration:{value:1200,timescale:600}} and
    (if $mode == "plan" then $item.source.content.text else $item.source.text end) ==
      "音频总线路由与侧链压低进入解析计划"
  ' "$doc" >/dev/null || fail "audio-routing-ducking $label Chinese label contract failed"
}

workflow_routing_input_contract() {
  local plan=$1 identity=$2 tone music key
  tone=$(workflow_material_id "$identity" tone)
  music=$(canonical_clip_id "$identity" music); key=$(canonical_clip_id "$identity" key)
  jq -e --arg tone "$tone" --arg music "$music" --arg key "$key" '
    def clip($id): first(.sequences[].tracks[].clips[] | select(.id == $id));
    [.inputs[] | select(.material_id == $tone)] as $inputs |
    ($inputs | length) == 1 and $inputs[0].canonical_uri == "assets/tone.wav" and
    $inputs[0].kind == {material_kind:"audio",type:"media"} and
    $inputs[0].audio.codec == "pcm_s16le" and
    $inputs[0].audio.info == {channel_layout:"unknown",channels:1,sample_rate:48000} and
    $inputs[0].audio.selection == {global_index:0,type_index:0} and
    $inputs[0].video == null and clip($music).source.input_id == $inputs[0].id and
    clip($key).source.input_id == $inputs[0].id
  ' "$plan" >/dev/null || fail "audio-routing-ducking resolved input contract failed"
}

#!/usr/bin/env bash

workflow_edit_contract() {
  local doc=$1 mode=$2 identity=$3 label=$4 first second text linked_video linked_audio trim remove sequence
  first=$(canonical_clip_id "$identity" first); second=$(canonical_clip_id "$identity" second)
  text=$(canonical_clip_id "$identity" explanation)
  linked_video=$(canonical_clip_id "$identity" linked-video)
  linked_audio=$(canonical_clip_id "$identity" linked-audio)
  trim=$(canonical_clip_id "$identity" trimmable); remove=$(canonical_clip_id "$identity" removable)
  sequence=$(canonical_sequence_id "$identity" main)
  jq -e --arg mode "$mode" --arg first "$first" --arg second "$second" --arg text "$text" \
    --arg linked_video "$linked_video" --arg linked_audio "$linked_audio" \
    --arg trim "$trim" --arg remove "$remove" --arg sequence "$sequence" '
    def clips: if $mode == "plan" then .sequences[].tracks[].clips[]
      else .project.sequences[].tracks[].clips[] end;
    def one($id): first(clips | select(.id == $id));
    one($first) as $first | one($second) as $second | one($text) as $text |
    $first.record_range == {start:{value:0,timescale:600},duration:{value:1200,timescale:600}} and
    $second.record_range == {start:{value:1200,timescale:600},duration:{value:1200,timescale:600}} and
    $first.source == {generator:{color:{alpha:255,blue:235,green:99,red:37},type:"solid"},
      type:"generated"} and
    $second.source == {generator:{color:{alpha:255,blue:119,green:39,red:219},type:"solid"},
      type:"generated"} and
    $text.record_range == {start:{value:0,timescale:600},duration:{value:2400,timescale:600}} and
    (if $mode == "plan" then $text.source.content.text else $text.source.text end) ==
      "编辑操作：分组移动、独立裁剪、音视频同时分割与删除" and
    (if $mode == "plan" then
      ([clips | .id]) as $rendered_ids |
      ($rendered_ids | index($linked_video)) == null and
      ($rendered_ids | index($linked_audio)) == null and
      ($rendered_ids | index($trim)) == null and
      ($rendered_ids | index($remove)) == null and
      ([.sequences[].tracks[] | select(.kind == "audio") | .clips[]] | length) == 0
    else
      one($linked_video).source == {generator:{type:"transparent"},type:"generated"} and
      one($linked_audio).source == {generator:{type:"silence"},type:"generated"} and
      one($trim).source == {generator:{type:"transparent"},type:"generated"} and
      one($remove).source == {generator:{type:"transparent"},type:"generated"} and
      all([one($linked_video),one($linked_audio),one($trim),one($remove)][];
        .record_range == {start:{value:0,timescale:600},duration:{value:2400,timescale:600}}) and
      ([.project.relations[] | select(.kind.type == "group")] | length) == 1 and
      first(.project.relations[] | select(.kind.type == "group")) as $group |
      $group.sequence_id == $sequence and ($group.kind.members | map(.item_id)) ==
        [$first.id,$second.id] and
      ([.project.relations[] | select(.kind.type == "av_link")] | length) == 1 and
      first(.project.relations[] | select(.kind.type == "av_link")) as $link |
      $link.sequence_id == $sequence and $link.kind.video.item_id == $linked_video and
      ($link.kind.audio | map(.item_id)) == [$linked_audio]
    end)
  ' "$doc" >/dev/null || fail "edit-operations $label typed contract failed"
}

workflow_probe_contract() {
  local doc=$1 mode=$2 identity=$3 label=$4 clip text material
  clip=$(canonical_clip_id "$identity" selected-stream)
  text=$(canonical_clip_id "$identity" explanation); material=$(workflow_material_id "$identity" source)
  jq -e --arg mode "$mode" --arg clip "$clip" --arg text "$text" --arg material "$material" '
    def clips: if $mode == "plan" then .sequences[].tracks[].clips[]
      else .project.sequences[].tracks[].clips[] end;
    def one($id): first(clips | select(.id == $id));
    one($clip) as $clip | one($text) as $text |
    $clip.record_range == {start:{value:0,timescale:600},duration:{value:1800,timescale:600}} and
    $text.record_range == $clip.record_range and
    (if $mode == "plan" then
      $clip.source.type == "media" and $clip.source.audio_stream == null and
      $clip.source.video_stream == {global_index:0,type_index:0} and
      ([.inputs[] | select(.material_id == $material)] | length) == 1 and
      first(.inputs[] | select(.material_id == $material)).canonical_uri == "assets/source.mp4" and
      first(.inputs[] | select(.material_id == $material)).audio == null and
      first(.inputs[] | select(.material_id == $material)).video.selection ==
        {global_index:0,type_index:0} and
      $clip.source.input_id == first(.inputs[] | select(.material_id == $material)).id and
      $text.source.content.text == "探测快照后显式选择全局零号视频流，并禁用音频流"
    else
      $clip.source == {material_id:$material,type:"media"} and
      ([.project.materials[] | select(.id == $material)] | length) == 1 and
      first(.project.materials[] | select(.id == $material)).kind == "video" and
      first(.project.materials[] | select(.id == $material)).source ==
        {type:"file",uri:"assets/source.mp4"} and
      first(.project.materials[] | select(.id == $material)).stream_intent ==
        {audio:{type:"disabled"},video:{global_index:0,type:"global_index"}} and
      $text.source.text == "探测快照后显式选择全局零号视频流，并禁用音频流"
    end)
  ' "$doc" >/dev/null || fail "probe-stream-selection $label typed contract failed"
}

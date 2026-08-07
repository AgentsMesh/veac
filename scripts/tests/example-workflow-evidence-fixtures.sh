#!/usr/bin/env bash

write_workflow_authoring_fixture() {
  local file=$1
  cat >"$file" <<'JSON'
{"project":{"revision":0,"timebase":600,"applied_operations":[],
"materials":[{"id":"med_source","kind":"video","identity":{"algorithm":"sha256","digest":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"},"source":{"type":"file","uri":"assets/source.mp4"},"stream_intent":{"video":{"type":"global_index","global_index":0},"audio":{"type":"disabled"}},"authorship":{"logical_path":["probe-stream-selection","resource","source"]}}],
"sequences":[{"tracks":[
{"id":"trk_scenes","clips":[
{"id":"itm_first","record_range":{"start":{"value":0,"timescale":600},"duration":{"value":1200,"timescale":600}},"authorship":{"logical_path":["edit-operations","item","first"]}},
{"id":"itm_second","record_range":{"start":{"value":1200,"timescale":600},"duration":{"value":1200,"timescale":600}},"authorship":{"logical_path":["edit-operations","item","second"]}}]},
{"id":"trk_trim","clips":[{"id":"itm_trim","record_range":{"start":{"value":0,"timescale":600},"duration":{"value":2400,"timescale":600}},"authorship":{"logical_path":["edit-operations","item","trimmable"]}}]},
{"id":"trk_video","clips":[{"id":"itm_video","record_range":{"start":{"value":0,"timescale":600},"duration":{"value":2400,"timescale":600}},"authorship":{"logical_path":["edit-operations","item","linked-video"]}}]},
{"id":"trk_audio","clips":[{"id":"itm_audio","record_range":{"start":{"value":0,"timescale":600},"duration":{"value":2400,"timescale":600}},"authorship":{"logical_path":["edit-operations","item","linked-audio"]}}]},
{"id":"trk_remove","clips":[{"id":"itm_remove","record_range":{"start":{"value":0,"timescale":600},"duration":{"value":2400,"timescale":600}},"authorship":{"logical_path":["edit-operations","item","removable"]}}]}]}],
"relations":[
{"id":"rel_group","kind":{"type":"group","members":[{"type":"item","item_id":"itm_first"},{"type":"item","item_id":"itm_second"}]}},
{"id":"rel_link","kind":{"type":"av_link","video":{"type":"item","item_id":"itm_video"},"audio":[{"type":"item","item_id":"itm_audio"}]}}]}}
JSON
}

write_edit_outcome_fixtures() {
  local authoring=$1 batch=$2 outcome=$3 replay=$4
  jq --slurpfile batch "$batch" '
    ($batch[0]) as $request |
    (.project.revision = 1) |
    (.project.applied_operations =
      [{id:$request.operation_id,request_hash:"fixture"}]) |
    (.project.sequences[].tracks[].clips[] |
      select(.id == "itm_first").record_range.start.value) = 300 |
    (.project.sequences[].tracks[].clips[] |
      select(.id == "itm_second").record_range.start.value) = 1500 |
    (.project.sequences[].tracks[].clips[] |
      select(.id == "itm_trim").record_range.duration.value) = 2100 |
    (.project.sequences[].tracks[] | select(.id == "trk_video").clips) |=
      (.[0].record_range.duration.value = 1200 |
       . + [(.[0] | .id = "itm_workflow_linked_video_right" |
         .record_range.start.value = 1200)]) |
    (.project.sequences[].tracks[] | select(.id == "trk_audio").clips) |=
      (.[0].record_range.duration.value = 1200 |
       . + [(.[0] | .id = "itm_workflow_linked_audio_right" |
         .record_range.start.value = 1200)]) |
    (.project.sequences[].tracks[] | select(.id == "trk_remove").clips) = [] |
    .project.relations += [{"id":"rel_workflow_linked_av_right",
      "kind":{"type":"av_link",
        "video":{"type":"item","item_id":"itm_workflow_linked_video_right"},
        "audio":[{"type":"item","item_id":"itm_workflow_linked_audio_right"}]}}] |
    . as $project |
    {status:"applied",new_revision:1,project:$project,
     normalized_operations:$request.operations,
     changed_objects:(["itm_first","itm_second","itm_trim","itm_video","itm_audio",
       "itm_remove","itm_workflow_linked_video_right",
       "itm_workflow_linked_audio_right"] | map({type:"item",id:.}))}
  ' "$authoring" >"$outcome"
  jq '{status:"no_change",current_revision:.new_revision,
    operation_recorded:true,project:.project}' "$outcome" >"$replay"
}

write_probe_evidence_fixtures() {
  local snapshot=$1 plan=$2
  cat >"$snapshot" <<'JSON'
{"schema_version":3,"engine":"ffprobe fixture","selection_policy":"veac.default-stream.v1","container_format":"mov,mp4","container_duration":{"value":1800,"timescale":600},"observed_identity":{"algorithm":"sha256","digest":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"},"streams":[{"global_index":0,"type_index":0,"media_type":"video","video":{"width":640,"height":360},"audio":null},{"global_index":1,"type_index":0,"media_type":"audio","video":null,"audio":{"sample_rate":48000,"channels":1}}],"selected_video_stream":{"global_index":0,"type_index":0},"selected_audio_stream":null}
JSON
  cat >"$plan" <<'JSON'
{"inputs":[{"material_id":"med_source","canonical_uri":"assets/source.mp4","observed_identity":{"algorithm":"sha256","digest":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"},"video":{"selection":{"global_index":0,"type_index":0}},"audio":null,"probe":{"schema_version":3,"engine":"ffprobe fixture","selection_policy":"veac.default-stream.v1","container_format":"mov,mp4","container_duration":{"value":1800,"timescale":600}}}]}
JSON
}

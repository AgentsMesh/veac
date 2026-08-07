#!/usr/bin/env bash

prepare_executable_fixture() {
  local id=$2 dir="$1/$2"
  mkdir -p "$dir/project/assets" "$dir/plans/preview" "$dir/rendered"
  printf '%s\n' "$dir"
}

write_executable_project() {
  local spec=$1 file=$2 id=$3 width=$4 height=$5 fps=$6
  jq --arg id "$id" --argjson width "$width" --argjson height "$height" \
    --argjson fps "$fps" '
    . as $spec | {project:{id:("prj_"+$id),entry_sequence_id:"seq_main",
      authorship:{entity:($spec.project_entity // null),deliveries:[{render_config_id:"out_preview",
        entity:{logical_path:[$id,"delivery","preview"]}}]},
      materials:$spec.materials,relations:($spec.relations // []),
      sequences:[{id:"seq_main",name:($spec.name // $id),
        settings:{width:$width,height:$height,sample_rate:48000,
          frame_rate:{numerator:$fps,denominator:1}},
        authorship:{entity:($spec.sequence_entity // null),tracks:($spec.track_authorship // []),
          relations:($spec.relation_authorship // [])},tracks:$spec.tracks}],
      render_configs:[{id:"out_preview",sequence_id:"seq_main",
        raster:{width:$width,height:$height,frame_rate:{numerator:$fps,denominator:1}},
        deliverables:[{id:"dlv_preview",target:{type:"file",name:"preview.mp4"},
          kind:{type:"video"}}]}]}}
  ' "$spec" >"$file"
}

write_executable_plan() {
  local spec=$1 file=$2 duration=$3 width=$4 height=$5 captions=$6 audio=$7
  jq --argjson duration "$duration" --argjson width "$width" \
    --argjson height "$height" --arg captions "$captions" --argjson audio "$audio" '
    . as $spec | {header:{schema_version:6,source:{timebase:600}},
      entry_sequence_id:"seq_main",inputs:($spec.inputs // []),
      output:{render_config_id:"out_preview",sequence_id:"seq_main",
        raster:{width:$width,height:$height,frame_rate:{numerator:12,denominator:1},
          captions:$captions},deliverables:[{kind:{settings:{audio:$audio}}}]},
      sequences:[{id:"seq_main",duration:{timescale:600,value:($duration*600)},
        tracks:$spec.tracks}],temporal:($spec.temporal //
          {bindings:[],programs:[],provenance:[],opset_version:1})}
  ' "$spec" >"$file"
}

copy_executable_preview() {
  local author=$1 preview=$2
  jq '.project.sequences[0].settings.frame_rate.numerator=12' "$author" >"$preview"
}

copy_fixture_tree() {
  local source=$1 target=$2 id=$3
  mkdir -p "$target"
  cp -R "$source/$id" "$target/$id"
}

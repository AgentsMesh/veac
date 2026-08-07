#!/usr/bin/env bash

write_stabilization_canonical() {
  local file=$1
  cat >"$file" <<'JSON'
{"project":{"sequences":[{"id":"seq_main","tracks":[{"clips":[
{"id":"itm_stabilize-before","record_range":{"start":{"timescale":1000,"value":4000},"duration":{"timescale":1000,"value":2000}},"source":{"type":"media","material_id":"med_shaky"},"source_mapping":{"time_map":{"source_start":{"timescale":1000,"value":0}}},"effects":[]},
{"id":"itm_stabilize-after","record_range":{"start":{"timescale":1000,"value":6000},"duration":{"timescale":1000,"value":2000}},"source":{"type":"media","material_id":"med_shaky"},"source_mapping":{"time_map":{"source_start":{"timescale":1000,"value":0}}},"effects":[{"effect":{"type":"video_stabilize","enabled":true},"enable_range":null,"enabled":true,"id":"fx_stabilize"}]}
]}]}],"render_configs":[{"id":"out_preview","sequence_id":"seq_main","deliverables":[{"id":"dlv_preview","target":{"type":"file","name":"preview.mp4"},"kind":{"type":"video","settings":{"video":{"codec":"h264"}}}}]}]}}
JSON
  jq '
    .project.entry_sequence_id = "seq_main" |
    .project.sequences[0].settings = {width:640,height:360} |
    .project.render_configs[0].raster = {width:640,height:360} |
    .project.authorship = {entity:{logical_path:["video-effects"],events:[]},
      multicam_groups:[],annotations:[],deliveries:[{render_config_id:"out_preview",
      entity:{logical_path:["video-effects","preview"],events:[]}}]} |
    .project.sequences[0].authorship = {type:"veac",
      entity:{logical_path:["video-effects","main"],events:[]},tracks:[],relations:[],applies:[]} |
    (.project.sequences[].tracks[].clips[]) |=
      (.authorship = {logical_path:
        ["video-effects","main","visual",(.id | sub("^itm_"; ""))],events:[]})
  ' "$file" >"$file.tmp"
  mv "$file.tmp" "$file"
}

make_stabilization_part() {
  local file=$1 x=$2 y=$3 content=${4:-dynamic} source
  if [[ $content == dynamic ]]; then
    source='testsrc2=s=520x310:r=12:d=2,drawgrid=w=36:h=36:t=2:c=white'
  else
    source='color=c=0x263247:s=520x310:r=12:d=2,drawgrid=w=36:h=36:t=2:c=white'
  fi
  ffmpeg -nostdin -v error -y -f lavfi \
    -i "$source" \
    -vf "crop=480:270:x='$x':y='$y'" -c:v libx264 -preset ultrafast -pix_fmt yuv420p "$file"
}

make_stabilization_video() {
  local dir=$1 mode=${2:-valid} parts
  parts="$dir/rendered/parts"
  local before_x='20+12*sin(n*1.7)' before_y='20+9*cos(n*1.3)'
  local after_x='20+1.5*sin(n*1.7)' after_y='20+1.5*cos(n*1.3)' after_content=dynamic
  case $mode in
    same-jitter) after_x=$before_x; after_y=$before_y ;;
    stable-before) before_x=$after_x; before_y=$after_y ;;
    frozen-after) after_x=20; after_y=20; after_content=static ;;
    frozen-shifted) after_content=static ;;
    missing-grid) before_x=20; before_y=20; after_x=20; after_y=20 ;;
  esac
  mkdir -p "$parts"
  if [[ $mode == missing-grid ]]; then
    ffmpeg -nostdin -v error -y -f lavfi -i 'color=c=0x263247:s=480x270:r=12:d=2' \
      -c:v libx264 -preset ultrafast -pix_fmt yuv420p "$parts/before.mp4"
    cp "$parts/before.mp4" "$parts/after.mp4"
  else
    make_stabilization_part "$parts/before.mp4" "$before_x" "$before_y"
    make_stabilization_part "$parts/after.mp4" "$after_x" "$after_y" "$after_content"
  fi
  ffmpeg -nostdin -v error -y -f lavfi -i 'color=c=0x18202e:s=480x270:r=12:d=4' \
    -i "$parts/before.mp4" -i "$parts/after.mp4" \
    -filter_complex '[0:v][1:v][2:v]concat=n=3:v=1:a=0[v]' -map '[v]' \
    -c:v libx264 -preset ultrafast -pix_fmt yuv420p "$dir/rendered/preview.mp4"
  rm -rf "$parts"
}

make_stabilization_fixture() {
  local dir="$1/video-effects" author preview plan
  mkdir -p "$dir/project" "$dir/plans/preview" "$dir/rendered"
  printf 'video stabilization fixture\n' >"$dir/project/main.veac"
  author=$(example_authoring_canonical "$dir"); preview=$(example_preview_canonical "$dir")
  plan=$(example_preview_plan "$dir" out_preview)
  write_stabilization_canonical "$author"
  jq '.project.render_configs[0].raster={width:480,height:270}' "$author" >"$preview"
  jq '{sequences:.project.sequences,output:{id:"pout_preview",render_config_id:"out_preview",
      sequence_id:"seq_main",raster:{width:480,height:270}}}
    | (.. | objects | select(.id? == "itm_stabilize-before" or .id? == "itm_stabilize-after")) |=
      (.source={audio_stream:null,input_id:"pin_shaky",type:"media",
        video_stream:{global_index:0,type_index:0}} |
       .source_mapping={frame_synthesis:"nearest",out_of_range:"strict",time_map:{direction:"forward",
        rate:{denominator:1,numerator:1},repeat:1,source_range_per_repeat:{start:{timescale:1000,value:0},
        duration:{timescale:1000,value:2000}},type:"linear"}})
    | (.. | objects | select(.id? == "itm_stabilize-after") | .effects[0]) |=
      {active_range:{start:{timescale:1000,value:0},duration:{timescale:1000,value:2000}},
       effect:.effect,id:.id}' \
    "$preview" >"$plan"
  make_stabilization_video "$dir"
}

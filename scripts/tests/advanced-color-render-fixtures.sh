#!/usr/bin/env bash

write_advanced_color_canonical() {
  local file=$1
  cat >"$file" <<'JSON'
{"project":{"materials":[{"id":"med_cinematic","kind":"lut3d","source":{"type":"file","uri":"assets/cinematic.cube"}},{"id":"med_tone-curve","kind":"lut1d","source":{"type":"file","uri":"assets/tone-curve.cube"}}],"sequences":[{"id":"seq_color-card","tracks":[{"clips":[{"id":"itm_chart","source":{"generator":{"gradient":{"stops":[{"color":{"alpha":179,"blue":61,"green":33,"red":20}},{"color":{"alpha":179,"blue":255,"green":134,"red":58}},{"color":{"alpha":179,"blue":141,"green":77,"red":255}},{"color":{"alpha":179,"blue":102,"green":209,"red":255}}]}}}}]}]},{"id":"seq_main","tracks":[{"clips":[{"id":"itm_alpha-check","source":{"generator":{"gradient":{"stops":[{"color":{"alpha":255,"blue":5,"green":5,"red":5}},{"color":{"alpha":255,"blue":5,"green":5,"red":5}},{"color":{"alpha":255,"blue":250,"green":250,"red":250}},{"color":{"alpha":255,"blue":250,"green":250,"red":250}}]}}},"record_range":{"start":{"timescale":1000,"value":0},"duration":{"timescale":1000,"value":8000}}}]},{"clips":[
{"id":"itm_reference","record_range":{"start":{"timescale":1000,"value":0},"duration":{"timescale":1000,"value":1000}}},
{"id":"itm_hsl-only","record_range":{"start":{"timescale":1000,"value":1000},"duration":{"timescale":1000,"value":1000}},"visual":{"color_pipeline":{"stages":[{"type":"hsl","adjustment":{"hue_degrees":18,"lightness":0.06,"range":"red","saturation":0.3}}]}}},
{"id":"itm_curves-only","record_range":{"start":{"timescale":1000,"value":2000},"duration":{"timescale":1000,"value":1000}},"visual":{"color_pipeline":{"stages":[{"type":"curves","curves":{"blue":null,"green":null,"luma":{"interpolation":"monotonic","points":[{"input":0,"output":0.03},{"input":0.45,"output":0.52},{"input":1,"output":0.96}]},"red":{"interpolation":"monotonic","points":[{"input":0,"output":0},{"input":0.5,"output":0.56},{"input":1,"output":1}]}}}]}}},
{"id":"itm_wheels-only","record_range":{"start":{"timescale":1000,"value":3000},"duration":{"timescale":1000,"value":1000}},"visual":{"color_pipeline":{"stages":[{"type":"wheels","wheels":{"gain":{"blue":-0.01,"green":0.02,"red":0.05},"gamma":{"blue":0,"green":0,"red":0.03},"lift":{"blue":0.04,"green":0,"red":0}}}]}}},
{"id":"itm_full-grade","record_range":{"start":{"timescale":1000,"value":4000},"duration":{"timescale":1000,"value":2000}},"visual":{"color_pipeline":{"stages":[
{"type":"basic","adjustment":{"exposure_stops":0.35,"fade":0.04,"highlights":-0.2,"shadows":0.18,"temperature_kelvin":4800,"tint":0.08}},
{"type":"matrix","adjustment":{"matrix":[1.08,0.02,0,0.01,0.96,0.03,0,0.04,0.9],"offset":[0.01,0,0.02]}},
{"type":"hsl","adjustment":{"hue_degrees":12,"lightness":0.04,"range":"red","saturation":0.18}},
{"type":"curves","curves":{"blue":null,"green":null,"luma":{"interpolation":"monotonic","points":[{"input":0,"output":0.03},{"input":0.45,"output":0.52},{"input":1,"output":0.96}]},"red":null}},
{"type":"wheels","wheels":{"gain":{"blue":-0.01,"green":0.02,"red":0.05},"gamma":{"blue":0,"green":0,"red":0.03},"lift":{"blue":0.04,"green":0,"red":0}}},
{"type":"lut","application":{"interpolation":"tetrahedral","material_id":"med_cinematic"}}]}}},
{"id":"itm_tone-curve","record_range":{"start":{"timescale":1000,"value":6000},"duration":{"timescale":1000,"value":2000}},"visual":{"color_pipeline":{"stages":[{"type":"lut","application":{"interpolation":"linear","material_id":"med_tone-curve"}}]}}}]}]}],
"render_configs":[{"id":"out_preview","sequence_id":"seq_main","raster":{"width":480,"height":270,"frame_rate":{"numerator":12,"denominator":1},"captions":"discard"},"deliverables":[{"id":"dlv_preview","target":{"type":"file","name":"preview.mp4"},"kind":{"type":"video","settings":{"video":{"codec":"h264"}}}}]}]}}
JSON
}

make_advanced_color_video() {
  local dir=$1 mode=${2:-valid} reference='color=c=0x10192c:s=480x270:r=12:d=1,drawbox=x=0:y=135:w=iw:h=135:color=0x586275:t=fill'
  local colors=(0x4e5069 0x596172 0x665e79 0x987052 0x3d7f92)
  case $mode in
    same-stages) colors=(0x343e50 0x343e50 0x343e50 0x343e50 0x343e50) ;;
    opaque) reference='color=c=0x14213d:s=480x270:r=12:d=1' ;;
    bad-composite) reference='color=c=0x202020:s=480x270:r=12:d=1' ;;
  esac
  ffmpeg -nostdin -v error -y \
    -f lavfi -i "$reference" \
    -f lavfi -i "color=c=${colors[0]}:s=480x270:r=12:d=1" \
    -f lavfi -i "color=c=${colors[1]}:s=480x270:r=12:d=1" \
    -f lavfi -i "color=c=${colors[2]}:s=480x270:r=12:d=1" \
    -f lavfi -i "color=c=${colors[3]}:s=480x270:r=12:d=2" \
    -f lavfi -i "color=c=${colors[4]}:s=480x270:r=12:d=2" \
    -filter_complex '[0:v][1:v][2:v][3:v][4:v][5:v]concat=n=6:v=1:a=0,format=yuv420p[v]' \
    -map '[v]' -c:v libx264 -preset ultrafast "$dir/rendered/preview.mp4"
}

make_advanced_color_fixture() {
  local dir="$1/advanced-color" author preview plan
  mkdir -p "$dir/project" "$dir/plans/preview" "$dir/rendered"
  printf 'advanced color fixture\n' >"$dir/project/main.veac"
  printf 'advanced color preview fixture\n' >"$dir/project/main.preview.veac"
  author=$(example_authoring_canonical "$dir")
  preview=$(example_preview_canonical "$dir")
  plan=$(example_preview_plan "$dir" out_preview)
  write_advanced_color_canonical "$author"
  cp "$author" "$preview"
  jq '
    {sequences:.project.sequences,inputs:[
      {id:"pin_cinematic",material_id:"med_cinematic",canonical_uri:"assets/cinematic.cube",
       kind:{type:"resource",material_kind:"lut3d"},
       observed_identity:{algorithm:"sha256",digest:("0" * 64)}},
      {id:"pin_tone-curve",material_id:"med_tone-curve",canonical_uri:"assets/tone-curve.cube",
       kind:{type:"resource",material_kind:"lut1d"},
       observed_identity:{algorithm:"sha256",digest:("1" * 64)}}],
     output:{id:"pout_preview",render_config_id:"out_preview",sequence_id:"seq_main"}} |
    (.. | objects | select(.id? == "itm_full-grade") |
      .visual.color_pipeline.stages[] | select(.type == "lut") | .application) =
      {input_id:"pin_cinematic",interpolation:"tetrahedral",kind:"three_dimensional"} |
    (.. | objects | select(.id? == "itm_tone-curve") |
      .visual.color_pipeline.stages[0].application) =
      {input_id:"pin_tone-curve",interpolation:"linear",kind:"one_dimensional"}
  ' "$preview" >"$plan"
  make_advanced_color_video "$dir"
}

#!/usr/bin/env bash

write_agentsmesh_authoring() {
  local file=$1
  cat >"$file" <<'JSON'
{"project":{"sequences":[{"id":"seq_main","tracks":[{"clips":[
{"id":"itm_backdrop","record_range":{"start":{"timescale":1000,"value":0},"duration":{"timescale":1000,"value":15000}},"source":{"generator":{"gradient":{"end":{"x":1,"y":1},"start":{"x":0,"y":0},"stops":[{"color":{"alpha":255,"blue":67,"green":42,"red":16},"offset":0},{"color":{"alpha":255,"blue":110,"green":118,"red":15},"offset":0.55},{"color":{"alpha":255,"blue":94,"green":197,"red":34},"offset":1}],"type":"linear"}}}},
{"id":"itm_title","record_range":{"start":{"timescale":1000,"value":0},"duration":{"timescale":1000,"value":5000}},"source":{"type":"text","text":"AgentsMesh","style":{"animation":{"granularity":"grapheme","stagger":{"timescale":1000,"value":45},"reveal":{"keyframes":[{"interpolation":{"type":"ease_out"},"value":0.1},{"interpolation":{"type":"linear"},"time":{"timescale":1000,"value":900},"value":1}]},"opacity":{"keyframes":[{"interpolation":{"type":"ease_out"},"value":0.75},{"interpolation":{"type":"linear"},"time":{"timescale":1000,"value":700},"value":1}]},"transform":{"position_offset":{"keyframes":[{"interpolation":{"type":"ease_out"},"value":{"y":{"unit":"pixels","value":48}}},{"interpolation":{"type":"linear"},"time":{"timescale":1000,"value":900},"value":{"y":{"unit":"pixels","value":0}}}]},"scale":{"keyframes":[{"interpolation":{"type":"ease_out"},"value":{"x":0.5,"y":0.5}},{"interpolation":{"type":"linear"},"time":{"timescale":1000,"value":900},"value":{"x":1,"y":1}}]}}}}}},
{"id":"itm_promise","record_range":{"start":{"timescale":1000,"value":5000},"duration":{"timescale":1000,"value":10000}},"source":{"type":"text","text":"一次构建系统，持续交付类型化产品。"}},
{"id":"itm_caption","record_range":{"start":{"timescale":1000,"value":2000},"duration":{"timescale":1000,"value":10000}},"source":{"type":"caption","text":"视频编辑成为类型化、可审查的代码。","style":{"background":{"color":{"alpha":204,"blue":45,"green":31,"red":0},"padding_pixels":18}}}}]}]}],
"render_configs":[{"id":"out_preview","sequence_id":"seq_main","raster":{"captions":"burn_in","frame_rate":{"denominator":1,"numerator":30},"height":1080,"width":1920},"deliverables":[{"id":"dlv_preview","target":{"name":"preview.mp4","type":"file"},"kind":{"type":"video","settings":{"hardware":{"type":"auto"},"video":{"codec":"h264"},"audio":null}}}]}]}}
JSON
}

write_agentsmesh_preview() {
  jq '(.project.render_configs[0].raster) = {captions:"burn_in",frame_rate:{denominator:1,numerator:12},height:270,width:480}
    | .project.render_configs[0].deliverables[0].kind.settings.hardware = {type:"software"}' "$1" >"$2"
}

write_agentsmesh_plan() {
  jq 'first(.project.render_configs[0]) as $out |
    {output:{id:"pout_preview",render_config_id:$out.id,sequence_id:$out.sequence_id,
      raster:$out.raster,deliverables:$out.deliverables},
     sequences:[{id:"seq_main",duration:{timescale:1000,value:15000},tracks:[{clips:[
       {id:"itm_title",record_range:{start:{timescale:1000,value:0},duration:{timescale:1000,value:5000}},source:{content:{presentation:{style:{animation:.project.sequences[0].tracks[0].clips[1].source.style.animation}}}}},
       {id:"itm_promise",record_range:{start:{timescale:1000,value:5000},duration:{timescale:1000,value:10000}}},
       {id:"itm_caption",record_range:{start:{timescale:1000,value:2000},duration:{timescale:1000,value:10000}}}]}]}]}' "$1" >"$2"
}

agentsmesh_title_filter() {
  case $1 in
    static) printf '%s' "drawbox=x=120:y=101:w=240:h=28:color=white:t=fill:enable='lt(t,5)'" ;;
    no-scale) printf '%s' "drawbox=x=230:y=112:w=10:h=20:color=white:t=fill:enable='lt(t,.2)',drawbox=x=210:y=108:w=50:h=20:color=white:t=fill:enable='between(t,.2,.499)',drawbox=x=180:y=104:w=110:h=20:color=white:t=fill:enable='between(t,.5,.899)',drawbox=x=145:y=101:w=190:h=20:color=white:t=fill:enable='between(t,.9,1.304)',drawbox=x=120:y=101:w=240:h=20:color=white:t=fill:enable='between(t,1.305,4.999)'" ;;
    no-rise) printf '%s' "drawbox=x=230:y=120:w=10:h=8:color=white:t=fill:enable='lt(t,.2)',drawbox=x=210:y=120:w=50:h=12:color=white:t=fill:enable='between(t,.2,.499)',drawbox=x=180:y=120:w=110:h=18:color=white:t=fill:enable='between(t,.5,.899)',drawbox=x=145:y=120:w=190:h=24:color=white:t=fill:enable='between(t,.9,1.304)',drawbox=x=120:y=120:w=240:h=28:color=white:t=fill:enable='between(t,1.305,4.999)'" ;;
    incomplete) printf '%s' "drawbox=x=230:y=130:w=10:h=8:color=white:t=fill:enable='lt(t,.2)',drawbox=x=210:y=124:w=50:h=12:color=white:t=fill:enable='between(t,.2,.499)',drawbox=x=180:y=116:w=110:h=18:color=white:t=fill:enable='between(t,.5,.899)',drawbox=x=155:y=109:w=150:h=21:color=white:t=fill:enable='between(t,.9,1.199)',drawbox=x=145:y=106:w=190:h=24:color=white:t=fill:enable='between(t,1.2,1.599)',drawbox=x=120:y=101:w=240:h=28:color=white:t=fill:enable='between(t,1.6,4.999)'" ;;
    *) printf '%s' "drawbox=x=230:y=130:w=10:h=8:color=white:t=fill:enable='lt(t,.2)',drawbox=x=210:y=124:w=50:h=12:color=white:t=fill:enable='between(t,.2,.499)',drawbox=x=180:y=116:w=110:h=18:color=white:t=fill:enable='between(t,.5,.899)',drawbox=x=145:y=106:w=190:h=24:color=white:t=fill:enable='between(t,.9,1.304)',drawbox=x=120:y=101:w=240:h=28:color=white:t=fill:enable='between(t,1.305,4.999)'" ;;
  esac
}

make_agentsmesh_video() {
  local dir=$1 mode=${2:-valid} duration=15 title promise caption panel source filter
  title=$(agentsmesh_title_filter "$mode")
  promise="drawbox=x=150:y=108:w=180:h=24:color=0xd8f3ef:t=fill:enable='gte(t,5)'"
  caption="drawbox=x=150:y=225:w=180:h=7:color=white:t=fill:enable='between(t,2,11.999)'"
  panel="drawbox=x=145:y=220:w=190:h=24:color=0x001f2d@0.8:t=fill:enable='between(t,2,11.999)'"
  source="nullsrc=s=480x270:r=12:d=15,format=gbrp,geq=r='16+18*(X/W+Y/H)/2':g='42+155*(X/W+Y/H)/2':b='67+27*(X/W+Y/H)/2'"
  case $mode in
    overlap) promise="$promise,drawbox=x=120:y=72:w=240:h=28:color=white:t=fill:enable='gte(t,5)'" ;;
    no-promise) promise=null ;;
    off-center-promise) promise="drawbox=x=70:y=108:w=180:h=24:color=0xd8f3ef:t=fill:enable='gte(t,5)'" ;;
    no-caption) caption=null ;;
    off-center-caption) caption="drawbox=x=70:y=225:w=180:h=7:color=white:t=fill:enable='between(t,2,11.999)'" ;;
    no-panel) panel=null ;;
    late-caption) caption="drawbox=x=150:y=225:w=180:h=14:color=white:t=fill:enable='gte(t,2)'"; panel="drawbox=x=145:y=220:w=190:h=24:color=0x001f2d@0.8:t=fill:enable='gte(t,2)'" ;;
    solid) source='color=c=0x0f766e:s=480x270:r=12:d=15' ;;
    short) duration=12 ;;
  esac
  filter="$title,$promise,$panel,$caption"
  filter=${filter//,null/}; filter=${filter//null,/}
  ffmpeg -nostdin -v error -y -f lavfi -i "$source" -t "$duration" -vf "$filter" \
    -c:v libx264 -preset ultrafast -pix_fmt yuv420p "$dir/rendered/preview.mp4"
}

make_agentsmesh_fixture() {
  local dir="$1/agentsmesh-intro-15s" project="$1/agentsmesh-intro-15s/project"
  mkdir -p "$project" "$dir/plans/preview" "$dir/rendered"
  printf 'project agentsmesh fixture\n' >"$project/main.veac"
  printf 'project agentsmesh preview fixture\n' >"$project/main.preview.veac"
  write_agentsmesh_authoring "$project/project.veac.json"
  write_agentsmesh_preview "$project/project.veac.json" "$project/project.preview.veac.json"
  write_agentsmesh_plan "$project/project.preview.veac.json" "$dir/plans/preview/out_preview.json"
  make_agentsmesh_video "$dir" valid
}

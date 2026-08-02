#!/usr/bin/env bash

write_all_features_authoring() {
  local file=$1
  cat >"$file" <<'JSON'
{"project":{"materials":[
{"id":"med_footage","kind":"video","source":{"type":"file","uri":"assets/footage.mp4"}},
{"id":"med_music","kind":"audio","source":{"type":"file","uri":"assets/music.wav"}}],
"sequences":[{"id":"seq_main","tracks":[
{"id":"trk_picture","kind":"video","clips":[{"id":"itm_shot",
"record_range":{"start":{"timescale":1000,"value":0},"duration":{"timescale":1000,"value":8000}},
"source_mapping":{"time_map":{"type":"linear","source_start":{"timescale":1000,"value":2000},"rate":{"numerator":1,"denominator":1}}},
"visual":{"color_pipeline":{"input":{"matrix":"bt709","primaries":"bt709","range":"limited","transfer":"bt709"},
"working":{"matrix":"rgb","primaries":"bt709","range":"full","transfer":"linear"},
"output":{"matrix":"bt709","primaries":"bt709","range":"limited","transfer":"bt709"},
"stages":[{"type":"basic","adjustment":{"exposure_stops":0.1,"fade":0.01,"highlights":-0.04,"shadows":0.06,"temperature_kelvin":6600,"tint":0}}]}}}]},
{"id":"trk_graphics","kind":"visual","clips":[{"id":"itm_lower-third",
"record_range":{"start":{"timescale":1000,"value":1000},"duration":{"timescale":1000,"value":5000}},
"source":{"generator":{"color":{"alpha":221,"blue":94,"green":4,"red":3}}},
"visual":{"frame":{"fit":"fill","height":{"unit":"pixels","value":180},"width":{"unit":"pixels","value":1500}},
"placement":{"anchor":"bottom","inset":{"x":0,"y":80},"type":"anchor"},"opacity":{"type":"constant","value":0.95}}}]},
{"id":"trk_score","kind":"audio","clips":[{"id":"itm_music-item",
"record_range":{"start":{"timescale":1000,"value":0},"duration":{"timescale":1000,"value":8000}},
"audio":{"gain":{"type":"constant","value":0.5011872336272722},"pan":{"type":"constant","value":0},"muted":false,"normalize":true,"pitch_policy":"preserve",
"crossfade":{"curve":"equal_power","fade_in":{"timescale":1000,"value":250},"fade_out":{"timescale":1000,"value":500}},
    "processors":[{"id":"aud_voice-limiter","kind":{"attack_ms":1,"ceiling_db":-1,"release_ms":80,"type":"limiter"}}]}}]},
{"id":"trk_subtitles","kind":"caption","clips":[{"id":"itm_cue","record_range":{"start":{"timescale":1000,"value":0},"duration":{"timescale":1000,"value":8000}},
"source":{"text":"一种语言，一份类型化中间表示，一套渲染计划。","style":{"background":{"color":{"alpha":204,"blue":0,"green":0,"red":0},"padding_pixels":20}}}}]}]}],
"relations":[{"id":"rel_edit-unit","kind":{"members":[{"item_id":"itm_shot","type":"item"},{"item_id":"itm_music-item","type":"item"}],"type":"group"},"sequence_id":"seq_main"},
{"id":"rel_linked-av","kind":{"audio":[{"item_id":"itm_music-item","type":"item"}],"type":"av_link","video":{"item_id":"itm_shot","type":"item"}},"sequence_id":"seq_main"}],
"render_configs":[{"id":"out_master","sequence_id":"seq_main","raster":{"captions":"burn_in","frame_rate":{"denominator":1,"numerator":30},"height":1080,"width":1920},
"deliverables":[{"id":"dlv_master","target":{"name":"all-features.mp4","type":"file"},"kind":{"type":"video","settings":{"container":"mp4","hardware":{"type":"auto"},"optimize_for_streaming":true,"pass_mode":"single",
"video":{"codec":"h264","pixel_format":"yuv420p"},"audio":{"channels":2,"codec":"aac","sample_rate":48000}}}}]}]}}
JSON
}

write_all_features_preview() {
  jq '(.project.render_configs[0].raster) = {captions:"burn_in",frame_rate:{denominator:1,numerator:12},height:270,width:480}
    | .project.render_configs[0].deliverables[0].kind.settings.hardware = {type:"software"}' "$1" >"$2"
}

write_all_features_plan() {
  jq 'first(.project.render_configs[] | select(.id == "out_master")) as $out |
    {output:{id:"pout_master",render_config_id:$out.id,sequence_id:$out.sequence_id,
      raster:$out.raster,deliverables:$out.deliverables},sequences:.project.sequences}
    | .sequences[0].duration = {timescale:1000,value:8000}
    | (.. | objects | select(.id? == "itm_shot") | .source_mapping.time_map) |=
      (. + {source_range_per_repeat:{start:.source_start,duration:{timescale:1000,value:8000}}} | del(.source_start))
    | (.. | objects | select(.id? == "itm_cue") | .source) |=
      {type:"caption",speaker:null,content:{text:.text,
        presentation:{type:"styled",style:.style}}}' "$1" >"$2"
}

all_features_visual_filter() {
  local grade='eq=brightness=0.02:saturation=1.05'
  local panel="drawbox=x=60:y=205:w=360:h=45:color=0x03045e@0.95:t=fill:enable='between(t,1,5.999)'"
  local backing='drawbox=x=110:y=202:w=260:h=24:color=black@0.8:t=fill'
  local caption='drawbox=x=130:y=210:w=220:h=10:color=white:t=fill'
  case $1 in
    valid) printf '%s' "$grade,$panel,$backing,$caption" ;;
    neutral-grade) printf '%s' "$panel,$backing,$caption" ;;
    always-panel) printf '%s' "$grade,${panel%%:enable=*},$backing,$caption" ;;
    early-panel) printf '%s' "$grade,${panel/5.999/4.999},$backing,$caption" ;;
    no-caption) printf '%s' "$grade,$panel,$backing" ;;
    caption-gap) printf '%s' "$grade,$panel,$backing,$caption:enable='not(between(t,2,2.499))'" ;;
    late-background) printf '%s' "$grade,$panel,$backing:enable='between(t,1,5.999)',$caption" ;;
  esac
}

make_all_features_video() {
  local dir=$1 mode=${2:-valid} offset=2 duration=8 filter
  [[ $mode == wrong-mapping ]] && offset=3
  [[ $mode == truncated ]] && duration=4
  [[ $mode == wrong-mapping || $mode == truncated ]] && mode=valid
  filter=$(all_features_visual_filter "$mode")
  ffmpeg -nostdin -v error -y -ss "$offset" -i "$dir/project/assets/footage.mp4" \
    -f lavfi -i 'sine=frequency=660:sample_rate=48000:duration=8' -t "$duration" \
    -vf "scale=480:270,$filter" -af \
    'volume=0.5011872336,loudnorm=I=-16:LRA=11:TP=-1.5,alimiter=limit=0.89125:attack=1:release=80:level=false,afade=t=in:st=0:d=0.25,afade=t=out:st=7.5:d=0.5' \
    -c:v libx264 -preset ultrafast -pix_fmt yuv420p -c:a aac -ar 48000 -ac 2 \
    "$dir/rendered/all-features.mp4"
}

make_all_features_audio_variant() {
  local dir=$1 mode=$2 audio filter='loudnorm=I=-16:LRA=11:TP=-1.5,alimiter=limit=0.89125:level=false'
  local rate=48000 channels=2 tmp="$dir/rendered/audio-variant.mp4"
  case $mode in
    no-fade-in) filter="$filter,afade=t=out:st=7.5:d=0.5" ;;
    no-fade-out) filter="$filter,afade=t=in:st=0:d=0.25" ;;
    silence) audio='anullsrc=r=48000:cl=stereo:d=8'; filter=anull ;;
    wrong-format) audio='sine=frequency=660:sample_rate=44100:duration=8'; rate=44100; channels=1 ;;
    unnormalized) filter='volume=0.2,afade=t=in:st=0:d=0.25,afade=t=out:st=7.5:d=0.5' ;;
    *) return 2 ;;
  esac
  audio=${audio:-'sine=frequency=660:sample_rate=48000:duration=8'}
  ffmpeg -nostdin -v error -y -i "$dir/rendered/all-features.mp4" -f lavfi -i "$audio" \
    -map 0:v:0 -map 1:a:0 -t 8 -c:v copy -af "$filter" -c:a aac -ar "$rate" -ac "$channels" "$tmp"
  mv "$tmp" "$dir/rendered/all-features.mp4"
}

make_all_features_fixture() {
  local dir="$1/all-features" project="$1/all-features/project"
  mkdir -p "$project/assets" "$dir/plans/preview" "$dir/rendered"
  printf 'project all-features fixture\n' >"$project/main.veac"
  write_all_features_authoring "$project/project.veac.json"
  write_all_features_preview "$project/project.veac.json" "$project/project.preview.veac.json"
  write_all_features_plan "$project/project.preview.veac.json" "$dir/plans/preview/out_master.json"
  ffmpeg -nostdin -v error -y -f lavfi -i \
    'testsrc2=size=480x270:rate=12:duration=11,hue=h=t*29:s=1,drawgrid=w=40:h=30:color=white@0.2:t=1' \
    -c:v libx264 -preset ultrafast -pix_fmt yuv420p "$project/assets/footage.mp4"
  ffmpeg -nostdin -v error -y -f lavfi -i 'sine=frequency=660:sample_rate=48000:duration=8' \
    -c:a pcm_s16le "$project/assets/music.wav"
  make_all_features_video "$dir" valid
}

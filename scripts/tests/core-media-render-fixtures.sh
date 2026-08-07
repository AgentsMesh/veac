#!/usr/bin/env bash

make_minimal() {
  local root=$1 dir="$1/minimal"
  prepare_preview_fixture_dirs "$dir"
  ffmpeg -v error -f lavfi -i 'color=c=#0f766e:s=480x270:r=12:d=3' \
    -vf 'drawbox=x=85:y=112:w=310:h=46:color=0x07131d@0.8:t=fill,drawbox=x=130:y=125:w=220:h=16:color=white:t=fill' \
    -c:v libx264 -pix_fmt yuv420p "$dir/rendered/preview.mp4"
  cat >"$dir/project/project.veac.json" <<'JSON'
{"project":{"authorship":{"entity":{"logical_path":["minimal"],"events":[]},"multicam_groups":[],"annotations":[],"deliveries":[{"render_config_id":"out_preview","entity":{"logical_path":["minimal","preview"],"events":[]}}]},"sequences":[{"id":"seq_main","tracks":[{"id":"trk_content","kind":"visual","clips":[{"id":"itm_background","authorship":{"logical_path":["minimal","main","content","background"],"events":[]},"record_range":{"start":{"timescale":600,"value":0},"duration":{"timescale":600,"value":1800}},"source":{"type":"generated","generator":{"type":"solid","color":{"red":15,"green":118,"blue":110,"alpha":255}}}},{"id":"itm_label","authorship":{"logical_path":["minimal","main","content","label"],"events":[]},"record_range":{"start":{"timescale":600,"value":0},"duration":{"timescale":600,"value":1800}},"source":{"type":"text","text":"最小 VEAC：项目、时间线、轨道、条目与交付"}}]}]}],"render_configs":[{"id":"out_preview","raster":{"width":480,"height":270,"frame_rate":{"numerator":12,"denominator":1}},"deliverables":[{"target":{"type":"file","name":"preview.mp4"},"kind":{"type":"video","settings":{"audio":null}}}]}]}}
JSON
  complete_video_settings "$dir/project/project.veac.json"
  mirror_fixture_preview_canonical "$dir"
  write_smoke_plan "$dir/plans/preview/out_preview.json" out_preview 3000
}

make_transitions() {
  local root=$1 dir="$1/transitions"
  prepare_preview_fixture_dirs "$dir"
  ffmpeg -v error -f lavfi -i 'color=c=black:s=480x270:r=12:d=4' \
    -vf "format=gbrp,geq=r='if(lt(T,1.6),230,if(gt(T,2.0),69,230+(69-230)*(T-1.6)/.4))':g='if(lt(T,1.6),57,if(gt(T,2.0),123,57+(123-57)*(T-1.6)/.4))':b='if(lt(T,1.6),70,if(gt(T,2.0),157,70+(157-70)*(T-1.6)/.4))',drawbox=x=70:y=205:w=340:h=42:color=0x07131d@0.8:t=fill,drawbox=x=120:y=217:w=240:h=14:color=white:t=fill" \
    -c:v libx264 -pix_fmt yuv420p "$dir/rendered/preview.mp4"
  cat >"$dir/plans/preview/out_preview.json" <<'JSON'
{"output":{"render_config_id":"out_preview","sequence_id":"seq_main"},"sequences":[{"id":"seq_main","duration":{"timescale":1000,"value":4000},"tracks":[{"transitions":[{"alignment":"centered","cut_time":{"value":1800,"timescale":1000},"record_window":{"start":{"value":1600,"timescale":1000},"duration":{"value":400,"timescale":1000}},"outgoing_range":{"start":{"value":1600,"timescale":1000},"duration":{"value":400,"timescale":1000}},"incoming_range":{"start":{"value":0,"timescale":1000},"duration":{"value":400,"timescale":1000}},"kind":{"type":"dissolve"}}]}]}]}
JSON
  cat >"$dir/project/project.veac.json" <<'JSON'
{"project":{"authorship":{"entity":{"logical_path":["transitions"],"events":[]},"multicam_groups":[],"annotations":[],"deliveries":[{"render_config_id":"out_preview","entity":{"logical_path":["transitions","preview"],"events":[]}}]},"sequences":[{"id":"seq_main","tracks":[{"id":"trk_scenes","kind":"visual","clips":[{"id":"itm_first","authorship":{"logical_path":["transitions","main","scenes","first"],"events":[]},"record_range":{"start":{"timescale":1000,"value":0},"duration":{"timescale":1000,"value":2000}},"source":{"type":"generated","generator":{"type":"solid","color":{"red":230,"green":57,"blue":70,"alpha":255}}}},{"id":"itm_second","authorship":{"logical_path":["transitions","main","scenes","second"],"events":[]},"record_range":{"start":{"timescale":1000,"value":1600},"duration":{"timescale":1000,"value":2400}},"source":{"type":"generated","generator":{"type":"solid","color":{"red":69,"green":123,"blue":157,"alpha":255}}}},{"id":"itm_label","authorship":{"logical_path":["transitions","main","labels","explanation"],"events":[]},"source":{"type":"text","text":"居中叠化：两个真实画面在四百毫秒窗口内交叉混合"}}]}]}],"relations":[{"id":"rel_dissolve","kind":{"type":"transition","from":{"type":"item","item_id":"itm_first"},"to":{"type":"item","item_id":"itm_second"},"transition":{"duration":{"timescale":1000,"value":400},"alignment":"centered","kind":{"type":"dissolve"}}}}],"render_configs":[{"id":"out_preview","raster":{"width":480,"height":270,"frame_rate":{"numerator":12,"denominator":1}},"deliverables":[{"target":{"type":"file","name":"preview.mp4"},"kind":{"type":"video","settings":{"audio":null}}}]}]}}
JSON
  complete_video_settings "$dir/project/project.veac.json"
  mirror_fixture_preview_canonical "$dir"
}

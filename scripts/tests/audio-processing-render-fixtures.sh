#!/usr/bin/env bash

write_audio_processing_identity() {
  local dir=$1
  jq '.project.entry_sequence_id="seq_main" |
    .project.sequences=[{"id":"seq_main","tracks":[
      {"id":"trk_voice","kind":"audio","order":0,"routing":{"type":"main_mix"},"clips":[{
        "id":"itm_voice","authorship":{"logical_path":["audio-processing","main","voice"],"events":[]},
        "audio":{"gain":{"keyframes":[{"id":"kf_gain-start"},{"id":"kf_gain-end"}]},
          "pan":{"keyframes":[{"id":"kf_pan-start"},{"id":"kf_pan-end"}]},"processors":[
            {"id":"aud_eq","kind":{"bands":[{"id":"eqb_presence"}]}},{"id":"aud_hpf"},
            {"id":"aud_lpf"},{"id":"aud_compressor"},{"id":"aud_limiter"},
            {"id":"aud_gate"},{"id":"aud_loudness"}]}}]},
      {"id":"trk_music","kind":"audio","order":1,
        "routing":{"type":"audio_bus","bus_id":"bus_music-bus"},"clips":[{
          "id":"itm_music","authorship":{"logical_path":["audio-processing","main","music"],"events":[]},
          "effects":[{"id":"fx_normalize"}]}]},
      {"id":"trk_key","kind":"audio","order":2,"routing":{"type":"main_mix"},"clips":[{
        "id":"itm_key","authorship":{"logical_path":["audio-processing","main","key"],"events":[]}}]},
      {"id":"trk_label","kind":"visual","order":3,"clips":[
        {"id":"itm_voice-label","authorship":{"logical_path":["audio-processing","main","voice-label"],"events":[]},
          "record_range":{"start":{"timescale":600,"value":0},"duration":{"timescale":600,"value":1200}},
          "visual":{"placement":{"anchor":"bottom","inset":{"x":0,"y":48},"type":"anchor"}}},
        {"id":"itm_route-label","authorship":{"logical_path":["audio-processing","main","route-label"],"events":[]},
          "record_range":{"start":{"timescale":600,"value":1200},"duration":{"timescale":600,"value":1200}},
          "visual":{"placement":{"anchor":"bottom","inset":{"x":0,"y":48},"type":"anchor"}}}]}
    ]}] | .project.relations=[{"id":"rel_duck","kind":{"type":"sidechain",
      "key":{"type":"track","track_id":"trk_key"},
      "target":{"type":"item","item_id":"itm_music"}}}]' \
    "$dir/project/project.veac.json" >"$dir/project/project.tmp"
  mv "$dir/project/project.tmp" "$dir/project/project.veac.json"
  mirror_fixture_preview_canonical "$dir"
}

write_audio_processing_plan() {
  cat >"$1/plans/preview/out_preview.json" <<'JSON'
{"header":{"schema":"https://veac.dev/schemas/render-plan","schema_version":6,
  "source":{"timebase":600},"resolver":{"effect_registry_version":"veac-ir-effects-v2"}},
"entry_sequence_id":"seq_main","output":{"sequence_id":"seq_main","deliverables":[{
  "kind":{"type":"video","settings":{"audio":{"channels":2,"codec":"aac","sample_rate":48000}}}}]},
"sequences":[{"id":"seq_main","duration":{"timescale":600,"value":2400},"tracks":[
{"id":"trk_voice","kind":"audio","order":0,"source_order":0,"placement_mode":"free",
 "routing":{"audio":{"type":"main_mix"},"visual":null},
 "state":{"audio_enabled":true,"include_in_render":true,"visual_enabled":false},"transitions":[],"clips":[{
  "id":"itm_voice","record_range":{"start":{"timescale":600,"value":0},"duration":{"timescale":600,"value":1200}},
  "source":{"type":"media","input_id":"pin_tone","video_stream":null,"audio_stream":{"global_index":0,"type_index":0}},
  "source_mapping":{"frame_synthesis":"nearest","out_of_range":"strict","time_map":{"direction":"forward","rate":{"denominator":1,"numerator":1},"repeat":1,"source_range_per_repeat":{"start":{"timescale":600,"value":0},"duration":{"timescale":600,"value":1200}},"type":"linear"}},
  "audio":{"gain":{"type":"keyframes","keyframes":[
    {"id":"kf_gain-start","interpolation":{"type":"linear"},"time":{"timescale":600,"value":0},"value":0.35},
    {"id":"kf_gain-end","interpolation":{"type":"ease_out"},"time":{"timescale":600,"value":600},"value":0.9}]},
   "pan":{"type":"keyframes","keyframes":[
    {"id":"kf_pan-start","interpolation":{"type":"linear"},"time":{"timescale":600,"value":0},"value":-0.4},
    {"id":"kf_pan-end","interpolation":{"type":"ease_in_out"},"time":{"timescale":600,"value":600},"value":0.4}]},
   "muted":false,"normalize":false,"pitch_policy":"preserve","sidechain":null,
   "crossfade":{"curve":"equal_power","fade_in":{"timescale":600,"value":120},"fade_out":{"timescale":600,"value":120}},
   "processors":[
    {"id":"aud_eq","kind":{"bands":[{"frequency_hz":3000,"gain_db":2,"id":"eqb_presence","q":1.2}],"type":"parametric_eq"}},
    {"id":"aud_hpf","kind":{"frequency_hz":80,"poles":2,"q":0.7,"type":"high_pass"}},
    {"id":"aud_lpf","kind":{"frequency_hz":18000,"poles":2,"q":0.7,"type":"low_pass"}},
    {"id":"aud_compressor","kind":{"attack_ms":10,"knee_db":4,"makeup_gain_db":2,"mix":0.75,"ratio":3,"release_ms":120,"threshold_db":-18,"type":"compressor"}},
    {"id":"aud_limiter","kind":{"attack_ms":5,"ceiling_db":-1,"release_ms":100,"type":"limiter"}},
    {"id":"aud_gate","kind":{"attack_ms":5,"range_db":-30,"ratio":2,"release_ms":80,"threshold_db":-50,"type":"gate"}},
    {"id":"aud_loudness","kind":{"integrated_lufs":-16,"loudness_range_lu":8,"true_peak_dbtp":-1,"type":"loudness"}}]},"effects":[]}]},
{"id":"trk_music","kind":"audio","order":1,"source_order":1,"placement_mode":"free",
 "routing":{"audio":{"type":"bus","bus_id":"bus_music-bus"},"visual":null},
 "state":{"audio_enabled":true,"include_in_render":true,"visual_enabled":false},"transitions":[],"clips":[{
  "id":"itm_music","record_range":{"start":{"timescale":600,"value":1200},"duration":{"timescale":600,"value":1200}},
  "source":{"type":"media","input_id":"pin_tone","video_stream":null,"audio_stream":{"global_index":0,"type_index":0}},
  "source_mapping":{"frame_synthesis":"nearest","out_of_range":"strict","time_map":{"direction":"forward","rate":{"denominator":1,"numerator":1},"repeat":1,"source_range_per_repeat":{"start":{"timescale":600,"value":0},"duration":{"timescale":600,"value":1200}},"type":"linear"}},
  "audio":{"crossfade":{"curve":"linear","fade_in":{"timescale":600,"value":150},"fade_out":{"timescale":600,"value":150}},"gain":{"type":"constant","value":0.65},"muted":false,"normalize":false,"pan":{"type":"constant","value":-0.2},"pitch_policy":"follow_speed","processors":[],"sidechain":{"active_range":null,"attack_ms":10,"ratio":4,"relation_id":"rel_duck","release_ms":120,"source":{"track_id":"trk_key","type":"track"},"threshold_db":-24}},
  "effects":[{"active_range":{"start":{"timescale":600,"value":0},"duration":{"timescale":600,"value":1200}},"effect":{"target_lufs":-14,"type":"audio_normalize"},"id":"fx_normalize"}]}]},
{"id":"trk_key","kind":"audio","order":2,"source_order":2,"placement_mode":"free",
 "routing":{"audio":{"type":"main_mix"},"visual":null},
 "state":{"audio_enabled":true,"include_in_render":true,"visual_enabled":false},"transitions":[],"clips":[{
  "id":"itm_key","record_range":{"start":{"timescale":600,"value":1200},"duration":{"timescale":600,"value":1200}},
  "source":{"type":"media","input_id":"pin_tone","video_stream":null,"audio_stream":{"global_index":0,"type_index":0}},
  "source_mapping":{"frame_synthesis":"nearest","out_of_range":"strict","time_map":{"direction":"forward","rate":{"denominator":1,"numerator":1},"repeat":1,"source_range_per_repeat":{"start":{"timescale":600,"value":0},"duration":{"timescale":600,"value":1200}},"type":"linear"}},
  "audio":{"crossfade":{"curve":"exponential","fade_in":{"timescale":600,"value":90},"fade_out":{"timescale":600,"value":90}},"gain":{"type":"constant","value":0.5},"muted":false,"normalize":false,"pan":{"type":"constant","value":0.2},"pitch_policy":"preserve","processors":[],"sidechain":null},"effects":[]}]}]}]}
JSON
}

make_audio_processing_fixture() {
  local root=$1 final_phase=${2:-tone} dir="$1/audio-processing" phase2 left right
  local first='min(t/0.25\,1)*min((2-t)/0.25\,1)*between(t\,0\,2)'
  local second='min((t-2)/0.25\,1)*min((4-t)/0.25\,1)*between(t\,2\,4)'
  mkdir -p "$dir/rendered"
  if [[ $final_phase == tone ]]; then phase2="0.09*sin(2*PI*660*t)*$second"; else phase2=0; fi
  left="(0.10+0.04*t)*sin(2*PI*440*t)*if(lt(t\,1)\,1\,0.25)*$first+$phase2"
  right="(0.10+0.04*t)*sin(2*PI*440*t)*if(lt(t\,1)\,0.25\,1)*$first+$phase2"
  ffmpeg -v error -y -f lavfi -i 'color=c=#3f3f46:s=480x270:r=12:d=4' \
    -f lavfi -i "aevalsrc=$left|$right:s=48000:d=4:c=stereo" \
    -vf "drawbox=x=20:y=180:w=440:h=60:color=0x07131d:t=fill,drawbox=x=140:y=205:w=200:h=15:color=white:t=fill:enable='lt(t,2)',drawbox=x=100:y=205:w=280:h=15:color=white:t=fill:enable='gte(t,2)'" \
    -map 0:v:0 -map 1:a:0 -t 4 -c:v libx264 -pix_fmt yuv420p \
    -c:a aac -b:a 192k -ar 48000 -ac 2 "$dir/rendered/preview.mp4"
  write_audio_project "$dir" preview preview.mp4
  write_audio_processing_identity "$dir"
  write_audio_processing_plan "$dir"
}

#!/usr/bin/env bash

source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/example-preview-layout-fixtures.sh"
# shellcheck source=scripts/tests/all-features-render-fixtures.sh
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/all-features-render-fixtures.sh"

write_audio_project() {
  local dir=$1 delivery_id=$2 target_name=$3
  prepare_preview_fixture_dirs "$dir"
  cat >"$dir/project/project.veac.json" <<JSON
{"project":{"render_configs":[{"id":"out_$delivery_id","deliverables":[{"id":"dlv_$delivery_id","target":{"type":"file","name":"$target_name"},"kind":{"type":"video"}}]}]}}
JSON
  mirror_fixture_preview_canonical "$dir"
}

write_audio_plan() {
  cat >"$1/plans/preview/out_preview.json" <<'JSON'
{
  "output":{"deliverables":[{"kind":{"type":"video","settings":{"audio":{"codec":"aac","sample_rate":48000,"channels":2}}}}]},
  "sequences":[{"tracks":[
    {"id":"trk_dialogue","routing":{"audio":{"type":"bus","bus_id":"bus_dialogue-bus"}},"clips":[{
      "id":"itm_voiceover","record_range":{"start":{"timescale":1000,"value":0},"duration":{"timescale":1000,"value":2000}},
      "source_mapping":{"time_map":{"source_range_per_repeat":{"start":{"timescale":1000,"value":0}}}},
      "audio":{"gain":{"type":"keyframes","keyframes":[{"time":{"timescale":1000,"value":0}},{"time":{"timescale":1000,"value":1000}}]},"pan":{"value":-0.65},"normalize":true,"pitch_policy":"preserve","crossfade":{"curve":"equal_power"},"processors":[{"id":"aud_dialogue-hpf","kind":{"type":"high_pass"}},{"id":"aud_dialogue-eq","kind":{"type":"parametric_eq","bands":[{"id":"eqb_presence"}]}},{"id":"aud_dialogue-limiter","kind":{"type":"limiter"}}]},"effects":[]
    }]},
    {"id":"trk_music","routing":{"audio":{"type":"bus","bus_id":"bus_music-bus"}},"clips":[{
      "id":"itm_music-bed","audio":{"sidechain":{"relation_id":"rel_duck","source":{"type":"bus","bus_id":"bus_key-bus"}},"processors":[{"id":"aud_music-compressor","kind":{"type":"compressor"}}]},"effects":[]
    }]},
    {"id":"trk_key","routing":{"audio":{"type":"bus","bus_id":"bus_key-bus"}},"clips":[]},
    {"id":"trk_gated","clips":[{
      "id":"itm_gated-tone","record_range":{"start":{"timescale":1000,"value":2000},"duration":{"timescale":1000,"value":2000}},
      "source_mapping":{"time_map":{"rate":{"numerator":2,"denominator":1},"source_range_per_repeat":{"duration":{"timescale":1000,"value":4000}}}},
      "audio":{"pitch_policy":"follow_speed","crossfade":{"curve":"linear"},"processors":[{"id":"aud_noise-gate","kind":{"type":"gate"}}]},"effects":[]
    }]},
    {"id":"trk_normalized","clips":[{
      "id":"itm_normalized-tone","record_range":{"start":{"timescale":1000,"value":4000},"duration":{"timescale":1000,"value":2000}},
      "source_mapping":{"time_map":{"source_range_per_repeat":{"start":{"timescale":1000,"value":4000}}}},
      "audio":{"crossfade":{"curve":"exponential"},"processors":[{"id":"aud_target-loudness","kind":{"type":"loudness"}}]},"effects":[]
    }]},
    {"id":"trk_normalized-effect","clips":[{
      "id":"itm_normalized-effect-tone","record_range":{"start":{"timescale":1000,"value":6000},"duration":{"timescale":1000,"value":2000}},
      "source_mapping":{"time_map":{"source_range_per_repeat":{"start":{"timescale":1000,"value":6000}}}},
      "effects":[{"effect_type":"audio.normalize","parameters":{"target_lufs":{"type":"number","value":-18}}}]
    }]}
  ]}]
}
JSON
}

make_audio_processing_fixture() {
  local root=$1 final_phase=${2:-tone}
  local dir="$root/audio-processing" phase4 left right
  mkdir -p "$dir/rendered"
  if [[ $final_phase == tone ]]; then
    phase4='0.135*sin(2*PI*440*t)*gte(t\,6)'
  else
    phase4=0
  fi
  left="0.12*sin(2*PI*440*t)*lt(t\,2)+0.035*sin(2*PI*1200*t)*lt(t\,2)+0.12*sin(2*PI*880*t)*between(t\,2\,4)+0.17*sin(2*PI*660*t)*between(t\,4\,6)+$phase4"
  right="0.035*sin(2*PI*440*t)*lt(t\,2)+0.12*sin(2*PI*1200*t)*lt(t\,2)+0.12*sin(2*PI*880*t)*between(t\,2\,4)+0.17*sin(2*PI*660*t)*between(t\,4\,6)+$phase4"
  ffmpeg -v error -y -f lavfi -i 'testsrc2=size=480x270:rate=12:duration=8' \
    -f lavfi -i "aevalsrc=$left|$right:s=48000:d=8:c=stereo" \
    -map 0:v:0 -map 1:a:0 -t 8 -c:v libx264 -pix_fmt yuv420p \
    -c:a aac -b:a 192k -ar 48000 -ac 2 "$dir/rendered/preview.mp4"
  write_audio_project "$dir" preview preview.mp4
  write_audio_plan "$dir"
}

make_generated_audio_fixture() {
  local root=$1 signal=${2:-silence} dir="$1/generated-graphics" source
  mkdir -p "$dir/rendered"
  if [[ $signal == silence ]]; then
    source='anullsrc=r=48000:cl=stereo:d=8'
  else
    source='sine=frequency=440:sample_rate=48000:duration=8'
  fi
  ffmpeg -v error -y -f lavfi -i 'color=c=#203040:s=480x270:r=12:d=8' \
    -f lavfi -i "$source" -t 8 -c:v libx264 -pix_fmt yuv420p \
    -c:a aac -ar 48000 -ac 2 "$dir/rendered/preview.mp4"
  write_audio_project "$dir" preview preview.mp4
}

make_all_features_audio_fixture() {
  local root=$1 signal=${2:-tone}
  make_all_features_fixture "$root"
  if [[ $signal == silence ]]; then
    make_all_features_audio_variant "$root/all-features" silence
  fi
}

make_executable_audio_fixture() {
  local root=$1 signal=${2:-silence} dir="$1/executable-mechanisms" source
  mkdir -p "$dir/rendered"
  if [[ $signal == silence ]]; then
    source='anullsrc=r=48000:cl=mono:d=2'
  else
    source='sine=frequency=440:sample_rate=48000:duration=2'
  fi
  ffmpeg -v error -y -f lavfi -i 'color=c=#203040:s=96x54:r=30:d=2' \
    -f lavfi -i "$source" -t 2 -c:v libx264 -pix_fmt yuv420p \
    -c:a aac -ar 48000 -ac 1 "$dir/rendered/executable-mechanisms.mp4"
  write_audio_project "$dir" main executable-mechanisms.mp4
}

make_nested_multicam_audio_fixture() {
  local root=$1 signal=${2:-none} sync=${3:-audio} dir="$1/nested-and-multicam"
  mkdir -p "$dir/rendered"
  if [[ $signal == none ]]; then
    ffmpeg -v error -y -f lavfi -i 'color=c=#203040:s=480x270:r=12:d=6' \
      -c:v libx264 -pix_fmt yuv420p "$dir/rendered/preview.mp4"
  else
    ffmpeg -v error -y -f lavfi -i 'color=c=#203040:s=480x270:r=12:d=6' \
      -f lavfi -i 'sine=frequency=440:sample_rate=48000:duration=6' \
      -c:v libx264 -pix_fmt yuv420p -c:a aac "$dir/rendered/preview.mp4"
  fi
  write_audio_project "$dir" preview preview.mp4
  cat >"$dir/plans/preview/out_preview.json" <<JSON
{
  "sequences": [{
    "tracks": [{
      "clips": [{
        "id": "itm_interview-cut",
        "source": {
          "type": "multicam",
          "source": {
            "group_id": "mcg_interview",
            "sync": {"basis": "$sync", "reference_angle_id": "ang_host-angle"},
            "angles": [
              {
                "id": "ang_host-angle",
                "input_id": "pin_host-angle",
                "video_stream": {"global_index": 0, "type_index": 0},
                "audio_stream": null,
                "source_offset": {"timescale": 1000, "value": 0}
              },
              {
                "id": "ang_guest-angle",
                "input_id": "pin_guest-angle",
                "video_stream": {"global_index": 0, "type_index": 0},
                "audio_stream": null,
                "source_offset": {"timescale": 1000, "value": 120}
              }
            ],
            "switches": []
          }
        }
      }]
    }]
  }]
}
JSON
}

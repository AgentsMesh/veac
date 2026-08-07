#!/usr/bin/env bash

source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/example-preview-layout-fixtures.sh"
# shellcheck source=scripts/tests/all-features-render-fixtures.sh
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/all-features-render-fixtures.sh"
# shellcheck source=scripts/tests/audio-processing-render-fixtures.sh
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/audio-processing-render-fixtures.sh"

write_audio_project() {
  local dir=$1 delivery_id=$2 target_name=$3
  prepare_preview_fixture_dirs "$dir"
  cat >"$dir/project/project.veac.json" <<JSON
{"project":{"authorship":{"entity":{"logical_path":["fixture"],"events":[]},"multicam_groups":[],"annotations":[],"deliveries":[{"render_config_id":"out_$delivery_id","entity":{"logical_path":["fixture","$delivery_id"],"events":[]}}]},"render_configs":[{"id":"out_$delivery_id","deliverables":[{"id":"dlv_$delivery_id","target":{"type":"file","name":"$target_name"},"kind":{"type":"video"}}]}]}}
JSON
  mirror_fixture_preview_canonical "$dir"
}

make_generated_audio_fixture() {
  local root=$1 signal=${2:-silence} dir="$1/generated-graphics" source
  mkdir -p "$dir/rendered"
  if [[ $signal == silence ]]; then
    source='anullsrc=r=48000:cl=stereo:d=9'
  else
    source='sine=frequency=440:sample_rate=48000:duration=9'
  fi
  ffmpeg -v error -y -f lavfi -i 'color=c=#203040:s=480x270:r=12:d=9' \
    -f lavfi -i "$source" -t 9 -c:v libx264 -pix_fmt yuv420p \
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
  local root=$1 signal=${2:-none} dir="$1/executable-mechanisms"
  mkdir -p "$dir/rendered"
  if [[ $signal == none ]]; then
    ffmpeg -v error -y -f lavfi -i 'color=c=#203040:s=96x54:r=30:d=2' \
      -t 2 -c:v libx264 -pix_fmt yuv420p "$dir/rendered/preview.mp4"
  else
    ffmpeg -v error -y -f lavfi -i 'color=c=#203040:s=96x54:r=30:d=2' \
      -f lavfi -i 'sine=frequency=440:sample_rate=48000:duration=2' \
      -t 2 -c:v libx264 -pix_fmt yuv420p -c:a aac "$dir/rendered/preview.mp4"
  fi
  write_audio_project "$dir" preview preview.mp4
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
  jq '.project.sequences = [{id:"seq_main",tracks:[{clips:[{
      id:"itm_interview-cut",
      authorship:{logical_path:["nested-and-multicam","main","video","interview-cut"],events:[]},
      source:{group_id:"mcg_interview"}}]}]}]
    | .project.multicam_groups = [{id:"mcg_interview",
      sync:{basis:"audio",reference_angle_id:"ang_host-angle"},
      angles:[{id:"ang_host-angle"},{id:"ang_guest-angle"}]}]' \
    "$dir/project/project.veac.json" >"$dir/project/project.tmp"
  mv "$dir/project/project.tmp" "$dir/project/project.veac.json"
  mirror_fixture_preview_canonical "$dir"
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

#!/usr/bin/env bash

smoke_visible_ratio() {
  local video=$1 time=$2
  ffmpeg -v error -ss "$time" -i "$video" -frames:v 1 \
    -vf 'scale=64:36:flags=area,format=rgb24' -f rawvideo - |
    od -An -v -tu1 | awk '
      { for (i=1; i<=NF; i++) {
          rgb[channel++]=$i
          if (channel==3) {
            if (rgb[0]+rgb[1]+rgb[2] >= 60) visible++
            pixels++; channel=0
          }
        }
      }
      END { if (pixels > 0) print visible/pixels; else exit 1 }'
}

assert_smoke_visible() {
  local video=$1 duration=$2 label=$3 fraction time ratio
  for fraction in 0.1 0.5 0.9; do
    time=$(awk -v duration="$duration" -v fraction="$fraction" \
      'BEGIN { printf "%.6f", duration * fraction }')
    ratio=$(smoke_visible_ratio "$video" "$time") ||
      fail "$label cannot decode its ${fraction} frame"
    awk -v value="$ratio" 'BEGIN { exit !(value >= 0.01) }' ||
      fail "$label is black at its ${fraction} duration sample"
  done
}

smoke_audio_codec() {
  case $1 in
    aac|opus|flac) printf '%s\n' "$1" ;;
    pcm_s16_le) printf '%s\n' pcm_s16le ;;
    pcm_s24_le) printf '%s\n' pcm_s24le ;;
    pcm_s32_le) printf '%s\n' pcm_s32le ;;
    *) fail "unsupported canonical audio codec: $1" ;;
  esac
}

assert_smoke_audio() {
  local video=$1 codec=$2 sample_rate=$3 channels=$4 expected=$5 label=$6
  if [[ $codec == none ]]; then
    assert_stream_count "$video" a 0 "$label"
    return
  fi
  local actual_codec audio_duration
  actual_codec=$(smoke_audio_codec "$codec")
  assert_stream_count "$video" a 1 "$label"
  assert_stream_field "$video" a:0 codec_name "$actual_codec" "$label"
  assert_stream_field "$video" a:0 sample_rate "$sample_rate" "$label"
  assert_stream_field "$video" a:0 channels "$channels" "$label"
  audio_duration=$(smoke_decoded_audio_duration "$video" "$sample_rate" "$channels") ||
    fail "$label audio cannot be fully decoded"
  assert_smoke_duration "$audio_duration" "$expected" 0.08 "$label audio"
}

assert_smoke_video() {
  local video=$1 width=$2 height=$3 rate=$4 container=$5 video_codec=$6
  local pixel_format=$7 alpha=$8 audio_codec=$9 sample_rate=${10} channels=${11}
  local expected=${12} label=${13} duration tolerance
  smoke_assert_nonempty_file "$video" "$label media"
  assert_stream_count "$video" v 1 "$label"
  assert_stream_field "$video" v:0 width "$width" "$label"
  assert_stream_field "$video" v:0 height "$height" "$label"
  assert_stream_field "$video" v:0 r_frame_rate "$rate" "$label"
  assert_smoke_encoding "$video" "$container" "$video_codec" "$pixel_format" "$alpha" "$label"
  duration=$(probe_duration "$video")
  tolerance=$(smoke_tolerance "$rate")
  assert_smoke_duration "$duration" "$expected" "$tolerance" "$label"
  assert_smoke_video_interval "$video" "$expected" "$rate" "$label"
  assert_smoke_audio "$video" "$audio_codec" "$sample_rate" "$channels" "$expected" "$label"
  assert_smoke_visible "$video" "$expected" "$label"
  ffmpeg -nostdin -v fatal -xerror -i "$video" \
    -map 0:v:0 -map '0:a?' -f null - || fail "$label does not fully decode"
}

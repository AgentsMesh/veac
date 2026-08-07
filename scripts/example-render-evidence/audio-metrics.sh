#!/usr/bin/env bash

audio_filter_log() {
  local media=$1 start=$2 duration=$3 filter=$4
  ffmpeg -nostdin -hide_banner -nostats -loglevel info -i "$media" \
    -vn -af "atrim=start=$start:duration=$duration,asetpts=PTS-STARTPTS,$filter" \
    -f null - 2>&1
}

audio_volume_metrics() {
  local log
  log=$(audio_filter_log "$1" "$2" "$3" "${4:+$4,}volumedetect")
  awk '
    /mean_volume:/ { mean=$(NF-1) }
    /max_volume:/ { peak=$(NF-1) }
    END { if (mean != "" && peak != "") print mean, peak; else exit 1 }
  ' <<< "$log"
}

audio_mean_db() {
  local mean _
  read -r mean _ <<< "$(audio_volume_metrics "$@")"
  printf '%s\n' "$mean"
}

assert_db_between() {
  local value=$1 minimum=$2 maximum=$3 label=$4
  awk -v value="$value" -v minimum="$minimum" -v maximum="$maximum" \
    'BEGIN { exit !(value >= minimum && value <= maximum) }' \
    || fail "$label: ${value} dB is outside [$minimum, $maximum]"
}

assert_non_silent_window() {
  local level
  level=$(audio_mean_db "$1" "$2" "$3" "")
  assert_db_between "$level" -45 -1 "$4"
}

audio_band_db() {
  local media=$1 start=$2 duration=$3 frequency=$4 channel=${5:-mix} pan
  case "$channel" in
    left) pan='pan=mono|c0=c0' ;;
    right) pan='pan=mono|c0=c1' ;;
    mix) pan='pan=mono|c0=0.5*c0+0.5*c1' ;;
    *) fail "unknown audio analysis channel: $channel" ;;
  esac
  audio_mean_db "$media" "$start" "$duration" \
    "$pan,bandpass=f=$frequency:t=h:w=80"
}

assert_tone_dominates() {
  local target reference
  target=$(audio_band_db "$1" "$2" "$3" "$4")
  reference=$(audio_band_db "$1" "$2" "$3" "$5")
  awk -v target="$target" -v reference="$reference" -v gap="$6" \
    'BEGIN { exit !(target >= reference + gap) }' \
    || fail "$7: ${4}Hz=${target}dB, ${5}Hz=${reference}dB"
}

assert_channel_tone_bias() {
  local left right
  left=$(audio_band_db "$1" "$2" "$3" "$4" left)
  right=$(audio_band_db "$1" "$2" "$3" "$4" right)
  if [[ $5 == left ]]; then
    awk -v favored="$left" -v other="$right" -v gap="$6" \
      'BEGIN { exit !(favored >= other + gap) }'
  else
    awk -v favored="$right" -v other="$left" -v gap="$6" \
      'BEGIN { exit !(favored >= other + gap) }'
  fi || fail "$7: ${4}Hz left=${left}dB, right=${right}dB"
}

assert_channel_level_bias() {
  local left right favored other
  left=$(audio_mean_db "$1" "$2" "$3" 'pan=mono|c0=c0')
  right=$(audio_mean_db "$1" "$2" "$3" 'pan=mono|c0=c1')
  if [[ $4 == left ]]; then favored=$left; other=$right; else favored=$right; other=$left; fi
  awk -v favored="$favored" -v other="$other" -v gap="$5" \
    'BEGIN { exit !(favored >= other + gap) }' ||
    fail "$6: left=${left}dB right=${right}dB"
}

assert_audio_window_louder() {
  local louder quieter
  louder=$(audio_mean_db "$1" "$2" "$3" '')
  quieter=$(audio_mean_db "$1" "$4" "$5" '')
  awk -v louder="$louder" -v quieter="$quieter" -v gap="$6" \
    'BEGIN { exit !(louder >= quieter + gap) }' ||
    fail "$7: louder=${louder}dB quieter=${quieter}dB"
}

audio_ebur_metrics() {
  local log
  log=$(audio_filter_log "$1" "$2" "$3" 'ebur128=peak=true')
  awk '
    /Integrated loudness:/ { section="integrated"; next }
    section == "integrated" && $1 == "I:" { integrated=$2; section="" }
    /True peak:/ { section="peak"; next }
    section == "peak" && $1 == "Peak:" { peak=$2; section="" }
    END { if (integrated != "" && peak != "") print integrated, peak; else exit 1 }
  ' <<< "$log"
}

assert_loudness_target() {
  local integrated peak
  read -r integrated peak <<< "$(audio_ebur_metrics "$1" "$2" "$3")"
  [[ $integrated =~ ^-?[0-9]+([.][0-9]+)?$ && $peak =~ ^-?[0-9]+([.][0-9]+)?$ ]] \
    || fail "$8: invalid ebur128 metrics: $integrated / $peak"
  awk -v value="$integrated" -v target="$4" -v tolerance="$5" \
    'BEGIN { delta=value-target; if (delta<0) delta=-delta; exit !(delta<=tolerance) }' \
    || fail "$8: integrated ${integrated} LUFS misses ${4} +/- ${5}"
  assert_db_between "$peak" "$6" "$7" "$8 true peak"
}

assert_audio_stream_duration() {
  local duration
  duration=$(stream_field "$1" a:0 duration)
  awk -v value="$duration" -v expected="$2" -v tolerance="$3" \
    'BEGIN { delta=value-expected; if (delta<0) delta=-delta; exit !(delta<=tolerance) }' \
    || fail "$4: audio duration ${duration:-unknown}s misses ${2}s +/- ${3}s"
}

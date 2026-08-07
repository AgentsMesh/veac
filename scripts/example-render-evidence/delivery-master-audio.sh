#!/usr/bin/env bash

delivery_audio_rms() {
  ffmpeg -nostdin -hide_banner -nostats -loglevel info -i "$1" -vn \
    -af volumedetect -f null - 2>&1 |
    awk '/mean_volume:/ { print $(NF-1); exit }'
}

assert_delivery_audio_rms() {
  local label=$1 rms=$2
  [[ -n $rms && $rms != "-inf" ]] || fail "$label is silent"
  awk -v value="$rms" 'BEGIN { exit !(value > -45 && value < -2) }' ||
    fail "$label RMS is outside (-45, -2) dB: $rms"
}

check_delivery_audio() {
  local dir=$1 wav="$1/rendered/master.wav"
  local width height rate duration count start _cover_time _cover rms tolerance config canonical
  read -r width height rate duration count start _cover_time _cover < <(delivery_metadata "$dir")
  tolerance=$(awk -F/ '{ print $2/$1/2 }' <<<"$rate")
  canonical="$dir/project/project.veac.json"
  config=$(delivery_config_id "$canonical" master)
  jq -e --arg config "$config" '
    first(.project.render_configs[] | select(.id == $config)) as $config |
    first($config.deliverables[] | select(.kind.type == "audio_stem" and
      .target.name == "master.wav")) as $stem |
    $stem.target == {"type":"file","name":"master.wav"} and
    $stem.kind == {"type":"audio_stem","settings":{
      "source":{"type":"master"},"format":"wav",
      "audio":{"codec":"pcm_s24_le","sample_rate":48000,"channels":2}}}
  ' "$canonical" >/dev/null ||
    fail "delivery WAV master-source/PCM canonical contract failed"
  require_file "$wav"
  assert_duration_close "$wav" "$duration" "$tolerance" "delivery WAV"
  assert_stream_count "$wav" a 1 "delivery WAV"
  assert_stream_count "$wav" v 0 "delivery WAV"
  assert_stream_field "$wav" a:0 codec_name pcm_s24le "delivery WAV"
  assert_stream_field "$wav" a:0 bits_per_raw_sample 24 "delivery WAV"
  assert_stream_field "$wav" a:0 sample_rate 48000 "delivery WAV"
  assert_stream_field "$wav" a:0 channels 2 "delivery WAV"
  rms=$(delivery_audio_rms "$wav")
  assert_delivery_audio_rms "delivery PCM audio" "$rms"
}

delivery_audio_window() {
  awk -v duration="$1" 'BEGIN { print duration/10, duration*8/10 }'
}

delivery_audio_signature() {
  local media=$1 start=$2 duration=$3
  printf '%s ' "$(delivery_audio_rms "$media")"
  printf '%s ' "$(audio_band_db "$media" "$start" "$duration" 220)"
  printf '%s ' "$(audio_band_db "$media" "$start" "$duration" 440)"
  printf '%s\n' "$(audio_band_db "$media" "$start" "$duration" 660)"
}

assert_delivery_audio_matches() {
  local reference=$1 candidate=$2 duration=$3 label=$4 start window
  local rr rt r440 r660 cr ct c440 c660
  read -r start window < <(delivery_audio_window "$duration")
  read -r rr rt r440 r660 < <(delivery_audio_signature "$reference" "$start" "$window")
  read -r cr ct c440 c660 < <(delivery_audio_signature "$candidate" "$start" "$window")
  awk -v rr="$rr" -v rt="$rt" -v r440="$r440" -v r660="$r660" \
    -v cr="$cr" -v ct="$ct" -v c440="$c440" -v c660="$c660" '
      function abs(v) { return v < 0 ? -v : v }
      BEGIN { exit !(rt >= r440+12 && rt >= r660+12 &&
                      ct >= c440+12 && ct >= c660+12 &&
                      abs(rr-cr) <= 1.5 && abs(rt-ct) <= 2.5) }
    ' || fail "$label does not carry the same 220 Hz master mix: reference=$rr/$rt/$r440/$r660 candidate=$cr/$ct/$c440/$c660"
}

assert_delivery_audio_match() {
  local dir=$1 _width _height _rate duration _count _start _cover_time _cover
  read -r _width _height _rate duration _count _start _cover_time _cover < <(delivery_metadata "$dir")
  assert_delivery_audio_matches "$dir/rendered/master.wav" "$dir/rendered/master.mp4" \
    "$duration" "delivery MP4 audio"
}

#!/usr/bin/env bash

check_delivery_mp3_contract() {
  local canonical=$1
  jq -e '
    first(.project.render_configs[] | select(.id == "out_master")) as $config |
    first($config.deliverables[] | select(.id == "dlv_podcast")) as $podcast |
    $podcast.target == {"type":"file","name":"podcast.mp3"} and
    $podcast.kind.type == "audio_file" and
    $podcast.kind.settings.source == {"type":"master"} and
    $podcast.kind.settings.encoding == {"type":"mp3","settings":{
      "bitrate_bps":192000,"sample_rate_hz":48000,"channel_layout":"stereo"}}
  ' "$canonical" >/dev/null || fail "delivery MP3 canonical contract failed"
}

check_delivery_mp3() {
  local rendered="$1/rendered" canonical="$1/project/project.veac.json"
  local podcast="$rendered/podcast.mp3" master="$rendered/master.mp4" wav="$rendered/master.wav"
  local _width _height rate duration _count _start _cover_time _cover podcast_rms bitrate tolerance
  read -r _width _height rate duration _count _start _cover_time _cover < <(delivery_metadata "$1")
  tolerance=$(awk -F/ '{ print $2/$1 }' <<<"$rate")
  check_delivery_mp3_contract "$canonical"
  assert_delivery_regular_file "$podcast" "delivery MP3"
  assert_stream_count "$podcast" v 0 "delivery MP3"
  assert_stream_count "$podcast" a 1 "delivery MP3"
  assert_stream_field "$podcast" a:0 codec_name mp3 "delivery MP3"
  assert_stream_field "$podcast" a:0 sample_rate 48000 "delivery MP3"
  assert_stream_field "$podcast" a:0 channels 2 "delivery MP3"
  bitrate=$(stream_field "$podcast" a:0 bit_rate)
  assert_numeric_between "$bitrate" 191000 193000 "delivery MP3 bitrate"
  assert_duration_close "$podcast" "$duration" "$tolerance" "delivery MP3"
  assert_delivery_decodes "$podcast" "delivery MP3"

  podcast_rms=$(delivery_audio_rms "$podcast")
  assert_delivery_audio_rms "delivery MP3 audio" "$podcast_rms"
  assert_delivery_audio_matches "$wav" "$master" "$duration" "delivery MP4 audio"
  assert_delivery_audio_matches "$wav" "$podcast" "$duration" "delivery MP3 audio"
}

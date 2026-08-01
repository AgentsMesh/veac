#!/usr/bin/env bash

run_delivery_audio_contracts() {
  local root rendered

  root="$TMP_DIR/bad-mp3-bitrate"
  clone_delivery_fixture "$root"
  rendered="$root/delivery-formats/rendered"
  ffmpeg -nostdin -v error -y -i "$rendered/master.wav" -c:a libmp3lame \
    -b:a 96k -ar 48000 -ac 2 "$rendered/podcast.mp3"
  expect_typed_failure bad_mp3_bitrate "$root" check_delivery_mp3 "delivery MP3 bitrate"

  root="$TMP_DIR/bad-mp3-tone"
  clone_delivery_fixture "$root"
  rendered="$root/delivery-formats/rendered"
  ffmpeg -nostdin -v error -y -f lavfi \
    -i 'sine=frequency=440:sample_rate=48000:duration=3' \
    -c:a libmp3lame -b:a 192k -ar 48000 -ac 2 "$rendered/podcast.mp3"
  expect_typed_failure wrong_mp3_master_mix "$root" check_delivery_mp3 \
    "same 220 Hz master mix"

  root="$TMP_DIR/bad-mp4-tone"
  clone_delivery_fixture "$root"
  rendered="$root/delivery-formats/rendered"
  ffmpeg -nostdin -v error -y -i "$rendered/master.mp4" -f lavfi \
    -i 'sine=frequency=440:sample_rate=48000:duration=3' -map 0:v:0 -map 1:a:0 \
    -c:v copy -c:a aac -ar 48000 -ac 2 -t 3 -movflags +faststart "$rendered/master.tmp.mp4"
  mv "$rendered/master.tmp.mp4" "$rendered/master.mp4"
  expect_typed_failure wrong_mp4_master_mix "$root" assert_delivery_audio_match \
    "same 220 Hz master mix"

  root="$TMP_DIR/bad-wav-tone"
  clone_delivery_fixture "$root"
  rendered="$root/delivery-formats/rendered"
  ffmpeg -nostdin -v error -y -f lavfi \
    -i 'sine=frequency=440:sample_rate=48000:duration=3' \
    -c:a pcm_s24le -ar 48000 -ac 2 "$rendered/master.wav"
  expect_typed_failure wrong_wav_master_mix "$root" assert_delivery_audio_match \
    "same 220 Hz master mix"

  root="$TMP_DIR/wrong-wav-source-declaration"
  clone_delivery_fixture "$root"
  rewrite_delivery_json "$root/delivery-formats" \
    'walk(if type == "object" and .id? == "dlv_master-audio" then
      .kind.settings.source = {"type":"track","track_id":"trk_other"} else . end)'
  expect_typed_failure wrong_wav_source_declaration "$root" check_delivery_audio \
    "WAV master-source/PCM canonical contract"
}

#!/usr/bin/env bash

executable_audio_caption_project_contract() {
  local file=$1 tone=$2 intro=$3 audible=$4 silent=$5 tone_material=$6 font=$7
  jq -e --arg tone "$tone" --arg intro "$intro" --arg audible "$audible" \
    --arg silent "$silent" --arg tone_material "$tone_material" --arg font "$font" '
    def seconds: .value/.timescale;
    def clip($id): first(.project.sequences[].tracks[].clips[]|select(.id==$id));
    clip($tone) as $tone_clip | [clip($intro),clip($audible),clip($silent)] as $captions |
    first(.project.materials[]|select(.id==$tone_material)) as $tone_asset |
    first(.project.materials[]|select(.id==$font)) as $font_asset |
    ($tone_clip.record_range.start|seconds)==1 and
    ($tone_clip.record_range.duration|seconds)==2 and
    $tone_clip.source.material_id==$tone_material and
    $tone_clip.audio=={gain:{type:"constant",value:.7},pan:{type:"constant",value:0},
      muted:false,normalize:false,pitch_policy:"preserve",processors:[],
      crossfade:{fade_in:{timescale:600,value:60},fade_out:{timescale:600,value:60},curve:"equal_power"}} and
    [$captions[].record_range.start|seconds]==[0,1.5,3] and
    [$captions[].record_range.duration|seconds]==[1.5,1.5,1] and
    [$captions[].source.text]==["可执行音频与字幕","有声音轨：均衡功率淡入淡出","无声区间：字幕仍然保持"] and
    $captions[0].source.speaker=="小石" and
    all($captions[];.source.style.font.material_id==$font) and
    $tone_asset.kind=="audio" and $tone_asset.source.uri=="assets/tone.wav" and
    $tone_asset.identity.digest=="5c3aaa006e4c341feddf589f9eb0ba3b215c60353e491abf8f06f4bee4478cfc" and
    $font_asset.kind=="font" and $font_asset.source.uri=="assets/veac-example-zh.ttf" and
    $font_asset.identity.digest=="64afada094cf447d71fad4ee726799c4fa74917f7d9a1113ed7ea3c4234ff774"
  ' "$file" >/dev/null || fail "executable-audio-caption project contract failed: $file"
}

check_executable_audio_caption_evidence() {
  local dir="$PREVIEW_ROOT/executable-audio-caption"
  [[ -d $dir ]] || return 0
  local canonical preview plan video tone intro audible silent tone_material font
  IFS=$'\t' read -r canonical preview plan video < <(executable_preview_artifacts "$dir")
  tone=$(executable_clip_id "$canonical" tone tone)
  intro=$(executable_clip_id "$canonical" captions intro)
  audible=$(executable_clip_id "$canonical" captions audible)
  silent=$(executable_clip_id "$canonical" captions silent)
  tone_material=$(executable_material_id "$canonical" tone)
  font=$(executable_material_id "$canonical" example-font)
  executable_audio_caption_project_contract "$canonical" "$tone" "$intro" "$audible" \
    "$silent" "$tone_material" "$font"
  executable_audio_caption_project_contract "$preview" "$tone" "$intro" "$audible" \
    "$silent" "$tone_material" "$font"
  jq -e --arg tone "$tone" --arg intro "$intro" --arg audible "$audible" \
    --arg silent "$silent" --arg tone_material "$tone_material" --arg font "$font" '
    def clip($id): first(.sequences[].tracks[].clips[]|select(.id==$id));
    clip($tone) as $tone_clip | [clip($intro),clip($audible),clip($silent)] as $captions |
    (.sequences[0].duration.value/600)==4 and .output.raster.captions=="burn_in" and
    .output.deliverables[0].kind.settings.audio=={codec:"aac",sample_rate:48000,channels:2} and
    (first(.inputs[]|select(.material_id==$tone_material)).audio.info)==
      {sample_rate:48000,channels:1,channel_layout:"unknown"} and
    (first(.inputs[]|select(.material_id==$tone_material)).observed_identity.digest)==
      "5c3aaa006e4c341feddf589f9eb0ba3b215c60353e491abf8f06f4bee4478cfc" and
    (first(.inputs[]|select(.material_id==$font)).observed_identity.digest)==
      "64afada094cf447d71fad4ee726799c4fa74917f7d9a1113ed7ea3c4234ff774" and
    $tone_clip.source.input_id==(first(.inputs[]|select(.material_id==$tone_material)).id) and
    $tone_clip.source.audio_stream=={global_index:0,type_index:0} and
    $tone_clip.audio.crossfade.curve=="equal_power" and
    [$captions[].source.content.text]==["可执行音频与字幕","有声音轨：均衡功率淡入淡出","无声区间：字幕仍然保持"] and
    $captions[0].source.speaker=="小石" and
    all($captions[];.source.content.presentation.style.font.requested.material_id==$font)
  ' "$plan" >/dev/null || fail "executable-audio-caption plan contract failed"
  assert_executable_video "$video" 4 480 270 1 executable-audio-caption
  assert_stream_field "$video" a:0 sample_rate 48000 executable-audio-caption
  assert_stream_field "$video" a:0 channels 2 executable-audio-caption
  assert_audio_stream_duration "$video" 4 0.08 executable-audio-caption
  assert_executable_silence "$video" 0.1 0.7 "audio-caption leading silence"
  assert_non_silent_window "$video" 1.2 1.2 "audio-caption declared tone"
  assert_executable_silence "$video" 3.2 0.6 "audio-caption trailing silence"
  assert_unique_frames "$video" 3 0.5 2.25 3.5
}

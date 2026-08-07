#!/usr/bin/env bash

SMOKE_OBJECT_ID_PATTERN='^[a-z][a-z0-9_-]{0,127}$'

smoke_video_rows() {
  jq -er --arg config_pattern "$SMOKE_OBJECT_ID_PATTERN" '
    [.project.render_configs[] |
      .id as $config | .raster as $raster |
      .deliverables[] | select(.kind.type == "video") |
      if ($config|type) != "string" or
          ($config|test($config_pattern)|not) then error("unsafe render config id")
      elif .target.type != "file" or (.target.name|type) != "string" then
        error("video target is not a file")
      elif ($raster|type) != "object" then error("video raster is missing")
      elif ($raster.width|type) != "number" or $raster.width <= 0 or
          $raster.width != ($raster.width|floor) or
          ($raster.height|type) != "number" or $raster.height <= 0 or
          $raster.height != ($raster.height|floor) or
          ($raster.frame_rate.numerator|type) != "number" or
          $raster.frame_rate.numerator <= 0 or
          $raster.frame_rate.numerator != ($raster.frame_rate.numerator|floor) or
          ($raster.frame_rate.denominator|type) != "number" or
          $raster.frame_rate.denominator <= 0 or
          $raster.frame_rate.denominator != ($raster.frame_rate.denominator|floor) then
        error("video raster is invalid")
      elif (.kind.settings|type) != "object" or
          (.kind.settings|has("audio")|not) then error("video settings are incomplete")
      elif (.kind.settings.container|type) != "string" or
          (.kind.settings.video|type) != "object" or
          (.kind.settings.video.codec|type) != "string" or
          (.kind.settings.video.pixel_format|type) != "string" or
          (.kind.settings.video.alpha|type) != "string" then error("video encoding is incomplete")
      else .kind.settings.audio as $audio |
        [$config, .target.name, $raster.width, $raster.height,
         ($raster.frame_rate.numerator|tostring) + "/" +
           ($raster.frame_rate.denominator|tostring), .kind.settings.container,
         .kind.settings.video.codec, .kind.settings.video.pixel_format,
         .kind.settings.video.alpha] +
        (if $audio == null then ["none","none","none"]
         elif ($audio|type) == "object" and ($audio.codec|type) == "string" and
             ($audio.sample_rate|type) == "number" and $audio.sample_rate > 0 and
             ($audio.channels|type) == "number" and $audio.channels > 0 then
           [$audio.codec, ($audio.sample_rate|tostring), ($audio.channels|tostring)]
         else error("invalid video audio settings") end)
        | @tsv end] |
    if length > 0 then .[] else error("no file video deliverable") end
  ' "$1"
}

smoke_plan_duration() {
  local dir=$1 config=$2 plan
  plan=$(example_preview_plan "$dir" "$config") || fail "unsafe smoke config ID: $config"
  smoke_assert_regular_file "$plan" "resolved plan"
  jq -er --arg config "$config" '
    if .output.render_config_id != $config then error("render config mismatch")
    else .output.sequence_id as $sequence |
      [.sequences[] | select(.id == $sequence)] |
      if length != 1 then error("entry sequence mismatch")
      else .[0].duration as $duration |
        if ($duration.value|type) == "number" and
            ($duration.timescale|type) == "number" and $duration.timescale > 0 and
            $duration.value > 0
        then $duration.value / $duration.timescale
        else error("invalid sequence duration") end
      end
    end
  ' "$plan"
}

check_example_media_smoke() {
  local dir=$1 id canonical rows config name width height rate container video_codec
  local pixel alpha audio_codec sample channels expected count=0
  id=$(basename "$dir")
  smoke_assert_directory "$dir" "$id example"
  smoke_assert_directory "$dir/project" "$id project"
  smoke_assert_directory "$dir/rendered" "$id rendered output"
  canonical=$(example_preview_canonical "$dir")
  smoke_assert_regular_file "$canonical" "$id preview canonical project"
  rows=$(smoke_video_rows "$canonical") || fail "$id has no valid file video deliverable"
  while IFS=$'\t' read -r config name width height rate container video_codec \
      pixel alpha audio_codec sample channels; do
    [[ $config =~ $SMOKE_OBJECT_ID_PATTERN ]] || fail "$id has an unsafe config ID: $config"
    [[ $name =~ ^[A-Za-z0-9][A-Za-z0-9._-]*$ && $name != *..* ]] ||
      fail "$id has an unsafe video target: $name"
    expected=$(smoke_plan_duration "$dir" "$config") ||
      fail "$id cannot resolve planned duration for $config"
    assert_smoke_video "$dir/rendered/$name" "$width" "$height" "$rate" "$container" \
      "$video_codec" "$pixel" "$alpha" "$audio_codec" "$sample" "$channels" \
      "$expected" "$id/$name"
    count=$((count + 1))
  done <<< "$rows"
  ((count > 0)) || fail "$id has no video to audit"
}

check_all_example_media_smoke() {
  local catalog=$1 dir
  assert_smoke_roster "$PREVIEW_ROOT" "$catalog"
  for dir in "$PREVIEW_ROOT"/*/; do
    [[ $(basename "$dir") == .fixtures ]] && continue
    check_example_media_smoke "${dir%/}"
  done
}

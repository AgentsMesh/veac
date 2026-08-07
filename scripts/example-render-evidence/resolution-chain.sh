#!/usr/bin/env bash

assert_resolution_chain() {
  local dir=$1 plan=$2 video=$3 label=$4 author preview
  author=$(example_authoring_canonical "$dir")
  preview=$(example_preview_canonical "$dir")
  require_file "$author"
  require_file "$preview"
  jq -e '
    .project.entry_sequence_id as $entry |
    first(.project.sequences[] | select(.id == $entry)).settings as $settings |
    $settings.width == 640 and $settings.height == 360 and
    all(.project.render_configs[]; .raster.width == 640 and .raster.height == 360)
  ' "$author" >/dev/null || fail "$label authoring resolution contract failed"
  jq -e '
    .project.entry_sequence_id as $entry |
    first(.project.sequences[] | select(.id == $entry)).settings as $settings |
    $settings.width == 640 and $settings.height == 360 and
    all(.project.render_configs[]; .raster.width == 480 and .raster.height == 270)
  ' "$preview" >/dev/null || fail "$label preview resolution contract failed"
  jq -e '
    .output.sequence_id as $entry |
    first(.sequences[] | select(.id == $entry)).settings as $settings |
    $settings.width == 640 and $settings.height == 360 and
    .output.raster.width == 480 and .output.raster.height == 270
  ' "$plan" >/dev/null || fail "$label plan resolution contract failed"
  assert_stream_field "$video" v:0 width 480 "$label rendered resolution"
  assert_stream_field "$video" v:0 height 270 "$label rendered resolution"
}

#!/usr/bin/env bash

source "$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)/example-preview-layout.sh"
source "$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)/example-preview-delivery.sh"
# shellcheck source=example-render-evidence/resolution-chain.sh
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/resolution-chain.sh"

fail() {
  echo "render evidence check failed: $*" >&2
  exit 1
}

require_file() {
  local path=$1
  [[ -s $path ]] || fail "missing or empty artifact: $path"
}

canonical_clip_id() {
  local canonical=$1 logical_key=$2
  [[ $logical_key =~ ^[a-z][a-z0-9-]*$ ]] || fail "unsafe clip logical key: $logical_key"
  jq -er --arg key "$logical_key" '
    [.project.sequences[].tracks[].clips[] |
      select(.authorship.logical_path[-1] == $key) | .id] |
    if length == 1 then .[0] else error("clip logical key is not unique") end
  ' "$canonical" || fail "cannot resolve clip logical key: $logical_key"
}

canonical_sequence_id() {
  local canonical=$1 logical_key=$2
  [[ $logical_key =~ ^[a-z][a-z0-9-]*$ ]] || fail "unsafe sequence logical key: $logical_key"
  jq -er --arg key "$logical_key" '
    [.project.sequences[] |
      select(.authorship.type == "veac" and
        .authorship.entity.logical_path[-1] == $key) | .id] |
    if length == 1 then .[0] else error("sequence logical key is not unique") end
  ' "$canonical" || fail "cannot resolve sequence logical key: $logical_key"
}

delivery_plan_path() {
  local example_dir=$1 logical_key=$2 canonical config
  canonical=$(example_preview_canonical "$example_dir")
  require_file "$canonical"
  config=$(delivery_config_id "$canonical" "$logical_key") ||
    fail "cannot resolve delivery plan: $logical_key"
  example_preview_plan "$example_dir" "$config"
}

delivery_video_path() {
  local example_dir=$1 logical_key=$2 target_name=$3
  local canonical config file_name
  canonical=$(example_preview_canonical "$example_dir")
  [[ $target_name =~ ^[A-Za-z0-9][A-Za-z0-9._-]*$ ]] ||
    fail "unsafe video target basename: $target_name"
  [[ $target_name != *..* ]] || fail "unsafe video target basename: $target_name"
  require_file "$canonical"
  config=$(delivery_config_id "$canonical" "$logical_key") ||
    fail "cannot resolve delivery: $logical_key"
  file_name=$(jq -er --arg config "$config" --arg target "$target_name" '
    [.project.render_configs[]? | select(.id == $config)] as $configs
    | if ($configs | length) != 1 then
        error("expected exactly one render config")
      else
        [$configs[0].deliverables[]? |
          select(.kind.type == "video" and .target.type == "file" and
            .target.name == $target)] as $videos
        | if ($videos | length) != 1 then
            error("expected exactly one video deliverable")
          else $videos[0].target.name
          end
      end
    | select(type == "string" and length > 0)
  ' "$canonical") || fail "cannot resolve video target: $logical_key/$target_name"
  printf '%s/rendered/%s\n' "$example_dir" "$file_name"
}

probe_duration() {
  ffprobe -v error -show_entries format=duration -of default=nw=1:nk=1 "$1"
}

stream_count() {
  local file=$1
  local type=$2
  ffprobe -v error -select_streams "$type" -show_entries stream=index \
    -of csv=p=0 "$file" | awk 'NF { count += 1 } END { print count + 0 }'
}

stream_field() {
  local file=$1
  local selector=$2
  local field=$3
  ffprobe -v error -select_streams "$selector" -show_entries "stream=$field" \
    -of default=nw=1:nk=1 "$file" | head -n 1
}

assert_stream_count() {
  local file=$1
  local type=$2
  local expected=$3
  local label=$4
  local actual
  actual=$(stream_count "$file" "$type")
  [[ $actual == "$expected" ]] || fail "$label: expected $expected $type streams, got $actual"
}

assert_stream_field() {
  local file=$1
  local selector=$2
  local field=$3
  local expected=$4
  local label=$5
  local actual
  actual=$(stream_field "$file" "$selector" "$field")
  [[ $actual == "$expected" ]] || fail "$label: expected $field=$expected, got ${actual:-<empty>}"
}

assert_stream_field_one_of() {
  local file=$1 selector=$2 field=$3 label=$4 actual expected choices=
  shift 4
  (($# > 0)) || fail "$label: no expected $field values"
  actual=$(stream_field "$file" "$selector" "$field")
  for expected in "$@"; do
    [[ $actual == "$expected" ]] && return 0
    choices="${choices:+$choices or }$expected"
  done
  fail "$label: expected $field=$choices, got ${actual:-<empty>}"
}

assert_duration_close() {
  local file=$1
  local expected=$2
  local tolerance=$3
  local label=$4
  local duration
  duration=$(probe_duration "$file")
  awk -v value="$duration" -v expected="$expected" -v tolerance="$tolerance" \
    'BEGIN { delta = value - expected; if (delta < 0) delta = -delta; exit !(delta <= tolerance) }' \
    || fail "$label: expected ${expected}s +/- ${tolerance}s, got ${duration:-unknown}s"
}

video_contract() {
  local video=$1
  local minimum_duration=$2
  require_file "$video"
  ffprobe -v error -select_streams v:0 \
    -show_entries stream=width,height:format=duration -of json "$video" \
    | jq -e --argjson min "$minimum_duration" '
        (.streams | length) == 1
        and (.streams[0].width > 0)
        and (.streams[0].height > 0)
        and ((.format.duration | tonumber) >= $min)
      ' >/dev/null || fail "invalid video contract: $video"
}

audio_stream_contract() {
  local video=$1
  local minimum_duration=$2
  ffprobe -v error -select_streams a:0 \
    -show_entries stream=codec_name:format=duration -of json "$video" \
    | jq -e --argjson min "$minimum_duration" '
        (.streams | length) == 1
        and (.streams[0].codec_name == "aac")
        and ((.format.duration | tonumber) >= $min)
      ' >/dev/null || fail "invalid AAC stream contract: $video"
}

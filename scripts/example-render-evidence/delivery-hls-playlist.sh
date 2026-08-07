#!/usr/bin/env bash

hls_playlist_uris() {
  awk '
    { sub(/\r$/, "") }
    /^[[:space:]]*$/ { next }
    /^#/ {
      line=$0
      while (match(line, /URI="[^"]+"/)) {
        print substr(line, RSTART+5, RLENGTH-6)
        line=substr(line, RSTART+RLENGTH)
      }
      next
    }
    { print }
  ' "$1"
}

assert_hls_uri() {
  local uri=$1 label=$2
  [[ $uri =~ ^[A-Za-z0-9][A-Za-z0-9._-]*$ && $uri != *..* ]] \
    || fail "$label has an unsafe or non-local URI: $uri"
}

assert_hls_member() {
  local root=$1 uri=$2 label=$3
  assert_hls_uri "$uri" "$label"
  assert_delivery_regular_file "$root/$uri" "$label member $uri"
}

hls_variant_attributes() {
  local master=$1 wanted=$2
  awk -v wanted="$wanted" '
    { sub(/\r$/,"") }
    /^#EXT-X-STREAM-INF:/ { attributes=$0; next }
    $0 == wanted { print attributes; found=1; exit }
    END { if (!found) exit 1 }
  ' "$master"
}

hls_attribute_value() {
  local attributes=$1 wanted=$2
  awk -v wanted="$wanted" '
    function inspect(token, equals, name, value) {
      equals=index(token,"="); if (!equals) return
      name=substr(token,1,equals-1); value=substr(token,equals+1)
      if (name == wanted) {
        if (value ~ /^".*"$/) value=substr(value,2,length(value)-2)
        print value; found++
      }
    }
    {
      line=$0; sub(/^[^:]*:/,"",line); token=""; quoted=0
      for (cursor=1; cursor<=length(line); cursor++) {
        character=substr(line,cursor,1)
        if (character == "\"") quoted=!quoted
        if (character == "," && !quoted) { inspect(token); token="" }
        else token=token character
      }
      inspect(token)
    }
    END { if (found != 1) exit 1 }
  ' <<<"$attributes"
}

hls_peak_segment_bps() {
  local root=$1 playlist=$2 duration uri bytes rate peak=0 count=0
  while IFS=$'\t' read -r duration uri; do
    [[ -n $duration && -n $uri ]] || continue
    bytes=$(wc -c <"$root/$uri" | tr -d '[:space:]')
    rate=$(awk -v bytes="$bytes" -v duration="$duration" \
      'BEGIN { printf "%.0f\n", bytes*8/duration }')
    ((rate > peak)) && peak=$rate
    ((count += 1))
  done < <(awk '
    { sub(/\r$/,"") }
    /^#EXTINF:/ { duration=$0; sub(/^#EXTINF:/,"",duration); sub(/,$/,"",duration); next }
    !/^#/ && length && duration != "" { print duration "\t" $0; duration="" }
  ' "$playlist")
  ((count > 0)) || return 1
  printf '%d\n' "$peak"
}

assert_hls_master_variant() {
  local master=$1 uri=$2 width=$3 height=$4 _target=$5 max=$6 audio=$7 label=$8
  local attributes bandwidth resolution codecs ceiling peak
  attributes=$(hls_variant_attributes "$master" "$uri") ||
    fail "$label is absent from the HLS master playlist"
  resolution=$(hls_attribute_value "$attributes" RESOLUTION) ||
    fail "$label HLS RESOLUTION is missing or duplicated"
  codecs=$(hls_attribute_value "$attributes" CODECS) ||
    fail "$label HLS CODECS is missing or duplicated"
  [[ $resolution == "${width}x${height}" &&
     $codecs =~ ^avc1[.][0-9A-Fa-f]{6},mp4a[.]40[.]2$ ]] ||
    fail "$label HLS master attributes do not match its codecs and raster"
  bandwidth=$(hls_attribute_value "$attributes" BANDWIDTH) ||
    fail "$label HLS BANDWIDTH is missing or duplicated"
  [[ $bandwidth =~ ^[0-9]+$ ]] || fail "$label HLS BANDWIDTH is invalid"
  peak=$(hls_peak_segment_bps "$(dirname "$master")" "$(dirname "$master")/$uri") ||
    fail "$label HLS peak segment bandwidth is unavailable"
  ceiling=$(awk -v max="$max" -v audio="$audio" 'BEGIN { print int((max+audio)*1.2) }')
  assert_numeric_between "$bandwidth" "$peak" "$ceiling" "$label HLS BANDWIDTH"
}

assert_hls_segment_timing() {
  local playlist=$1 seconds=$2 label=$3
  awk -v expected="$seconds" '
    { sub(/\r$/,"") }
    /^#EXTINF:/ {
      value=$0; sub(/^#EXTINF:/,"",value)
      if (value !~ /^[0-9]+([.][0-9]+)?,$/) { bad=1; next }
      sub(/,$/,"",value)
      delta=value-expected; if (delta<0) delta=-delta
      if (delta>0.02) bad=1
      count++
    }
    END { exit !(count==3 && !bad) }
  ' "$playlist" || fail "$label must contain exactly three ${seconds}s segments"
}

assert_hls_exact_tag() {
  local playlist=$1 tag=$2 label=$3
  awk -v expected="$tag" '
    { sub(/\r$/,""); if ($0 == expected) found++ }
    END { exit !(found == 1) }
  ' "$playlist" || fail "$label must contain exactly one $tag tag"
}

assert_hls_segment_keyframe() {
  local segment=$1 label=$2 key
  key=$(ffprobe -v error -select_streams v:0 -show_frames \
    -show_entries frame=key_frame -of json "$segment" | jq -er '.frames[0].key_frame')
  [[ $key == 1 ]] || fail "$label does not start on an independent video keyframe"
}

assert_hls_segment_duration() {
  local segment=$1 expected=$2 label=$3 actual
  actual=$(stream_field "$segment" v:0 duration)
  awk -v actual="$actual" -v expected="$expected" '
    BEGIN { delta=actual-expected; if (delta<0) delta=-delta; exit !(delta<=0.09) }
  ' || fail "$label video duration is ${actual:-unknown}s, expected ${expected}s"
}

check_hls_closed_tree() {
  local root=$1 seconds=$2 master="$1/master.m3u8" uri playlist segment unexpected actual expected
  local media=() members=(master.m3u8) segments=()
  assert_delivery_regular_file "$master" "HLS master playlist"
  while IFS= read -r uri; do
    assert_hls_member "$root" "$uri" "HLS master playlist"
    media+=("$uri"); members+=("$uri")
  done < <(hls_playlist_uris "$master")
  ((${#media[@]} == 2)) || fail "HLS master must reference exactly two media playlists"
  for playlist in "${media[@]}"; do
    assert_hls_exact_tag "$root/$playlist" '#EXT-X-INDEPENDENT-SEGMENTS' \
      "HLS media playlist $playlist"
    assert_hls_exact_tag "$root/$playlist" '#EXT-X-PLAYLIST-TYPE:VOD' \
      "HLS media playlist $playlist"
    assert_hls_exact_tag "$root/$playlist" '#EXT-X-ENDLIST' \
      "HLS media playlist $playlist"
    assert_hls_segment_timing "$root/$playlist" "$seconds" "$playlist"
    while IFS= read -r segment; do
      assert_hls_member "$root" "$segment" "HLS media playlist $playlist"
      segments+=("$segment"); members+=("$segment")
    done < <(hls_playlist_uris "$root/$playlist")
  done
  unexpected=$(find "$root" -mindepth 1 -maxdepth 1 ! -type f -print -quit)
  [[ -z $unexpected ]] || fail "HLS package contains a non-regular member: $unexpected"
  expected=$(printf '%s\n' "${members[@]}" | LC_ALL=C sort -u)
  actual=$(find "$root" -mindepth 1 -maxdepth 1 -type f -exec basename {} \; | LC_ALL=C sort)
  [[ $actual == "$expected" ]] || fail "HLS package contains missing or unreachable members"
  for segment in "${segments[@]}"; do
    assert_delivery_decodes "$root/$segment" "HLS segment $segment"
    assert_hls_segment_duration "$root/$segment" "$seconds" "HLS segment $segment"
    assert_hls_segment_keyframe "$root/$segment" "HLS segment $segment"
  done
}

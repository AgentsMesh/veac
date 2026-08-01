#!/usr/bin/env bash

smoke_tolerance() {
  local rate=$1 numerator denominator
  numerator=${rate%/*}
  denominator=${rate#*/}
  [[ $rate =~ ^[1-9][0-9]*/[1-9][0-9]*$ ]] || fail "invalid frame rate: $rate"
  awk -v n="$numerator" -v d="$denominator" 'BEGIN { print 0.05 + 2*d/n }'
}

smoke_stream_tolerance() {
  local rate=$1 numerator denominator
  numerator=${rate%/*}
  denominator=${rate#*/}
  awk -v n="$numerator" -v d="$denominator" 'BEGIN { print 0.02 + d/n }'
}

assert_smoke_duration() {
  local actual=$1 expected=$2 tolerance=$3 label=$4
  awk -v a="$actual" -v e="$expected" -v t="$tolerance" '
    BEGIN { delta=a-e; if (delta<0) delta=-delta; exit !(a>0 && e>0 && delta<=t) }
  ' || fail "$label duration ${actual:-unknown}s does not match planned ${expected}s"
}

assert_smoke_zero() {
  local actual=$1 label=$2
  awk -v value="$actual" 'BEGIN {
    if (value < 0) value=-value
    exit !(value <= 0.001)
  }' || fail "$label must start at zero, got ${actual:-unknown}s"
}

smoke_decoded_audio_duration() {
  local bytes
  bytes=$(ffmpeg -nostdin -v fatal -i "$1" -map 0:a:0 \
    -acodec pcm_s16le -f s16le - | wc -c | tr -d ' ') || return 1
  [[ $bytes =~ ^[0-9]+$ ]] || return 1
  awk -v bytes="$bytes" -v rate="$2" -v channels="$3" \
    'BEGIN { if (rate <= 0 || channels <= 0) exit 1; print bytes/(2*rate*channels) }'
}

smoke_video_interval() {
  ffprobe -v fatal -select_streams v:0 -show_packets \
    -show_entries stream=start_time,duration:packet=pts_time,duration_time \
    -of json "$1" | jq -er '
      def number_or_null:
        if type == "number" then .
        elif type == "string" then try tonumber catch null
        else null end;
      if (.streams|length) != 1 then error("missing video stream") else . end |
      .streams[0] as $stream |
      [.packets[]? |
        (.pts_time|number_or_null) as $start |
        (.duration_time|number_or_null // 0) as $duration |
        select($start != null and $duration >= 0) |
        {start:$start, end:($start+$duration)}] as $packets |
      if ($packets|length) == 0 then error("missing video packet timestamps") else
        [($stream.start_time|number_or_null // "none"),
         ($stream.duration|number_or_null // "none"),
         ($packets|min_by(.start)|.start), ($packets|max_by(.end)|.end)] | @tsv
      end'
}

assert_smoke_video_interval() {
  local video=$1 expected=$2 rate=$3 label=$4 row
  local stream_start stream_duration packet_start packet_end tolerance
  row=$(smoke_video_interval "$video") || fail "$label video interval cannot be probed"
  IFS=$'\t' read -r stream_start stream_duration packet_start packet_end <<< "$row"
  [[ $stream_start == none ]] || assert_smoke_zero "$stream_start" "$label video stream"
  assert_smoke_zero "$packet_start" "$label first video packet"
  tolerance=$(smoke_stream_tolerance "$rate")
  [[ $stream_duration == none ]] ||
    assert_smoke_duration "$stream_duration" "$expected" "$tolerance" "$label video"
  assert_smoke_duration "$packet_end" "$expected" "$tolerance" "$label video endpoint"
}

#!/usr/bin/env bash

smoke_video_codec() {
  case $1 in
    h264|vp9|av1) printf '%s\n' "$1" ;;
    h265) printf '%s\n' hevc ;;
    pro_res) printf '%s\n' prores ;;
    dnx_hr) printf '%s\n' dnxhd ;;
    *) fail "unsupported canonical video codec: $1" ;;
  esac
}

smoke_ebml_doctype() {
  od -An -v -tu1 -N 4096 "$1" | awk '
    function vint(pos, keep, first, marker, vlen, i, unknown) {
      first=byte[pos]
      if (first >= 128) vlen=1
      else if (first >= 64) vlen=2
      else if (first >= 32) vlen=3
      else if (first >= 16) vlen=4
      else if (first >= 8) vlen=5
      else if (first >= 4) vlen=6
      else if (first >= 2) vlen=7
      else if (first >= 1) vlen=8
      else return 0
      if (pos + vlen - 1 > count) return 0
      marker=2 ^ (8-vlen); value=first; unknown=1
      if (!keep) {
        value-=marker
        if (value != marker-1) unknown=0
      }
      for (i=1; i<vlen; i++) {
        value=value*256+byte[pos+i]
        if (byte[pos+i] != 255) unknown=0
      }
      if (!keep && unknown) return 0
      width=vlen
      return 1
    }
    { for (i=1; i<=NF; i++) byte[++count]=$i }
    END {
      if (count < 5 || byte[1] != 26 || byte[2] != 69 ||
          byte[3] != 223 || byte[4] != 163 || !vint(5, 0)) exit 1
      cursor=5+width; header_end=cursor+value
      if (header_end > count+1) exit 1
      while (cursor < header_end) {
        if (!vint(cursor, 1)) exit 1
        id=value; cursor+=width
        if (!vint(cursor, 0)) exit 1
        size=value; cursor+=width; item_end=cursor+size
        if (item_end > header_end) exit 1
        if (id == 17026) {
          result=""
          for (i=cursor; i<item_end; i++) result=result sprintf("%c", byte[i])
          if (result == "matroska" || result == "webm") print result
          else exit 1
          exit
        }
        cursor=item_end
      }
      exit 1
    }'
}

assert_smoke_container() {
  local video=$1 container=$2 label=$3 probe doctype
  probe=$(ffprobe -v error -show_entries format=format_name:format_tags=major_brand \
    -of json "$video") || fail "$label container cannot be probed"
  case $container in
    mp4)
      jq -e '(.format.format_name|split(",")|index("mp4")) != null and
        ((.format.tags.major_brand // "") | startswith("qt  ") | not)' \
        <<<"$probe" >/dev/null || fail "$label is not an MP4 container" ;;
    mov)
      jq -e '(.format.tags.major_brand // "") | startswith("qt  ")' \
        <<<"$probe" >/dev/null || fail "$label is not a QuickTime MOV container" ;;
    mkv|webm)
      jq -e '.format.format_name == "matroska,webm"' \
        <<<"$probe" >/dev/null || fail "$label is not an EBML video container"
      doctype=$(smoke_ebml_doctype "$video") || fail "$label has no valid EBML DocType"
      [[ ($container == mkv && $doctype == matroska) ||
         ($container == webm && $doctype == webm) ]] ||
        fail "$label EBML DocType does not match $container" ;;
    mxf)
      jq -e '.format.format_name | startswith("mxf")' \
        <<<"$probe" >/dev/null || fail "$label is not an MXF container" ;;
    *) fail "unsupported canonical video container: $container" ;;
  esac
}

assert_smoke_encoding() {
  local video=$1 container=$2 codec=$3 pixel_format=$4 alpha=$5 label=$6 actual_codec
  actual_codec=$(smoke_video_codec "$codec")
  [[ $alpha == opaque || ($alpha == straight && $pixel_format == yuva444p10le) ]] ||
    fail "$label has an invalid canonical alpha contract"
  assert_smoke_container "$video" "$container" "$label"
  assert_stream_field "$video" v:0 codec_name "$actual_codec" "$label"
  assert_stream_field "$video" v:0 pix_fmt "$pixel_format" "$label"
}

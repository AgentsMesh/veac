#!/usr/bin/env bash
set -euo pipefail

SOURCE=${1:?source path required}
TMP=$(mktemp "${SOURCE}.XXXXXX")
trap 'rm -f "$TMP"' EXIT

awk '
  {
    lines[NR] = $0
    if ($0 ~ /font family "[^"]+";/) needs_font = 1
    if ($0 ~ /^[[:space:]]*output video[[:space:]]/) has_video = 1
    if ($0 ~ /^[[:space:]]*entry sequence[[:space:]]/) {
      entry = $3
      sub(/;$/, "", entry)
    }
  }
  END {
    inserted_font = 0
    for (i = 1; i < NR; i++) {
      line = lines[i]
      if (needs_font && !inserted_font && line ~ /^  (multicam|sequence|annotation|output)/) {
        print "  resource font preview-font {"
        print "    locator local { path \"assets/preview-font.ttf\"; }"
        print "  }"
        print ""
        inserted_font = 1
      }
      gsub(/font family "[^"]+";/, "font resource preview-font;", line)
      print line
    }
    if (!has_video) {
      print ""
      print "  output video preview {"
      print "    sequence " entry ";"
      print "    file-name \"preview.mp4\";"
      print "    encoding {"
      print "      container mp4; optimize-for-streaming true;"
      print "      video { codec h264; pixel-format yuv420p; }"
      print "      audio none;"
      print "      captions burn-in;"
      print "    }"
      print "  }"
    }
    print lines[NR]
  }
' "$SOURCE" >"$TMP"

mv "$TMP" "$SOURCE"
trap - EXIT

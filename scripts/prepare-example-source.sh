#!/usr/bin/env bash
set -euo pipefail

SOURCE=${1:?source path required}
OUTPUT=${2:?output path required}
[[ -f "$SOURCE" && ! -L "$SOURCE" ]] || {
  echo "preview source input must be a regular file: $SOURCE" >&2
  exit 1
}
[[ "$SOURCE" != "$OUTPUT" ]] || {
  echo "preview source output must differ from its input" >&2
  exit 1
}
[[ ! -e "$OUTPUT" && ! -L "$OUTPUT" ]] || {
  echo "preview source output already exists: $OUTPUT" >&2
  exit 1
}
TMP=$(mktemp "${OUTPUT}.XXXXXX")
trap 'rm -f "$TMP"' EXIT

awk '
  {
    lines[NR] = $0
    if ($0 ~ /font family "[^"]+";/) needs_font = 1
    if ($0 ~ /fallback-font family "[^"]+";/) needs_arabic_font = 1
    if ($0 ~ /^[[:space:]]*resource[[:space:]]+font[[:space:]]+("preview-font"|preview-font)[[:space:]]*\{/) {
      has_preview_font = 1
    }
    if ($0 ~ /^[[:space:]]*resource[[:space:]]+font[[:space:]]+("preview-arabic-font"|preview-arabic-font)[[:space:]]*\{/) {
      has_preview_arabic_font = 1
    }
  }
  END {
    inserted_font = 0
    for (i = 1; i < NR; i++) {
      line = lines[i]
      if (!inserted_font && line ~ /^  (multicam|sequence|annotation|delivery)/ &&
          ((needs_font && !has_preview_font) || (needs_arabic_font && !has_preview_arabic_font))) {
        if (needs_font && !has_preview_font) {
          print "  resource font preview-font {"
          print "    locator local { path \"assets/preview-font.ttf\"; }"
          print "  }"
          print ""
        }
        if (needs_arabic_font && !has_preview_arabic_font) {
          print "  resource font preview-arabic-font {"
          print "    locator local { path \"assets/preview-arabic-font.ttf\"; }"
          print "  }"
          print ""
        }
        inserted_font = 1
      }
      gsub(/fallback-font family "[^"]+";/, "fallback-font resource preview-arabic-font;", line)
      gsub(/font family "[^"]+";/, "font resource preview-font;", line)
      print line
    }
    print lines[NR]
  }
' "$SOURCE" >"$TMP"

mv "$TMP" "$OUTPUT"
trap - EXIT

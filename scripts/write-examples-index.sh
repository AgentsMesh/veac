#!/usr/bin/env bash
set -euo pipefail

OUTPUT=${1:?preview directory is required}
EXPECTED=${2:?expected example count is required}
TEMPORARY="$OUTPUT/index.html.tmp"
COUNT=0

[[ "$EXPECTED" =~ ^[1-9][0-9]*$ ]] || {
  echo "examples preview: invalid expected count: $EXPECTED" >&2
  exit 1
}

{
  cat <<'HTML'
<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>VEAC example previews</title>
  <style>
    * { box-sizing: border-box; }
    body { margin: 0; color: #20262e; background: #f3f5f7; font: 15px/1.5 system-ui, sans-serif; }
    header { padding: 24px max(20px, calc((100% - 1180px) / 2)); color: #f8fafb; background: #20262e; }
    h1 { margin: 0; font-size: 24px; letter-spacing: 0; }
    main { display: grid; grid-template-columns: repeat(auto-fit, minmax(min(100%, 340px), 1fr)); gap: 18px; max-width: 1180px; margin: 0 auto; padding: 24px 20px 40px; }
    article { min-width: 0; overflow: hidden; border: 1px solid #d7dce1; border-radius: 6px; background: #fff; }
    h2 { margin: 0; padding: 13px 14px; border-bottom: 1px solid #e2e6e9; font-size: 16px; letter-spacing: 0; }
    video, img { display: block; width: 100%; aspect-ratio: 16 / 9; background: #101419; object-fit: contain; }
    audio { display: block; width: calc(100% - 28px); margin: 20px 14px; }
    nav { display: flex; flex-wrap: wrap; gap: 12px; padding: 12px 14px; }
    a { color: #0b6e69; text-underline-offset: 3px; }
  </style>
</head>
<body>
  <header><h1>VEAC example previews</h1></header>
  <main>
HTML

  for entry in "$OUTPUT"/*; do
    [[ -d "$entry/rendered" ]] || continue
    name=$(basename "$entry")
    [[ "$name" =~ ^[a-z0-9][a-z0-9-]*$ ]] || continue
    [[ -s "$entry/plan.json" ]] || {
      echo "example has no primary render plan: $name" >&2
      exit 1
    }
    media_count=0
    printf '    <article>\n      <h2>%s</h2>\n' "$name"
    for media in "$entry"/rendered/*; do
      [[ -f "$media" ]] || continue
      file=$(basename "$media")
      [[ "$file" =~ ^[A-Za-z0-9._-]+$ ]] || {
        echo "unsafe preview filename: $file" >&2
        exit 1
      }
      case "${file##*.}" in
        mp4|mov|mkv|webm)
          printf '      <video controls preload="metadata" src="%s/rendered/%s"></video>\n' "$name" "$file" ;;
        m4a|mp3|ogg|wav)
          printf '      <audio controls preload="metadata" src="%s/rendered/%s"></audio>\n' "$name" "$file" ;;
        jpg|jpeg|png|webp)
          printf '      <img loading="lazy" src="%s/rendered/%s" alt="%s preview">\n' "$name" "$file" "$name" ;;
        *)
          printf '      <nav><a href="%s/rendered/%s">Artifact</a></nav>\n' "$name" "$file" ;;
      esac
      media_count=$((media_count + 1))
    done
    [[ $media_count -gt 0 ]] || {
      echo "example has no rendered preview: $name" >&2
      exit 1
    }
    printf '      <nav><a href="%s/project/main.veac">Source</a><a href="%s/project/project.veac.json">IR</a><a href="%s/plan.json">Plan</a><a href="%s/build.log">Log</a></nav>\n' "$name" "$name" "$name" "$name"
    printf '    </article>\n'
    COUNT=$((COUNT + 1))
  done

  cat <<'HTML'
  </main>
</body>
</html>
HTML
} > "$TEMPORARY"

[[ $COUNT -eq $EXPECTED ]] || {
  rm -f "$TEMPORARY"
  echo "examples preview: expected $EXPECTED entries, built $COUNT" >&2
  exit 1
}
mv "$TEMPORARY" "$OUTPUT/index.html"

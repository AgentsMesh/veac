#!/usr/bin/env bash
set -euo pipefail

IMAGE='ghcr.io/jrottenberg/ffmpeg@sha256:bd0ea8c9d115f9dd8d2d1369f036fbc3d0e49bb6989d48e776f45c2d5e2e93ba'
ROOT="${RUNNER_TEMP:?RUNNER_TEMP is required}/ffmpeg-8-root"
BIN="$RUNNER_TEMP/ffmpeg-8-bin"
container=''

cleanup() {
  [[ -z "$container" ]] || docker rm --force "$container" >/dev/null 2>&1 || true
}
trap cleanup EXIT

[[ ! -e "$ROOT" && ! -e "$BIN" ]] || {
  echo "FFmpeg install paths must start absent" >&2
  exit 1
}
mkdir -p "$ROOT" "$BIN"
container=$(docker create --platform linux/amd64 "$IMAGE" -version)
docker export "$container" | tar -xf - -C "$ROOT"
cleanup
container=''
trap - EXIT

for command in ffmpeg ffprobe; do
  wrapper="$BIN/$command"
  {
    printf '%s\n' '#!/bin/bash' 'set -euo pipefail'
    printf '%s\n' 'root="${RUNNER_TEMP:?RUNNER_TEMP is required}/ffmpeg-8-root"'
    printf 'exec "$root/lib/ld-musl-x86_64.so.1" --library-path "$root/lib:$root/usr/lib" "$root/bin/%s" "$@"\n' "$command"
  } >"$wrapper"
  chmod +x "$wrapper"
done

ffmpeg_version=$("$BIN/ffmpeg" -version)
ffprobe_version=$("$BIN/ffprobe" -version)
filters=$("$BIN/ffmpeg" -hide_banner -filters 2>&1)
grep -Eq '^ffmpeg version 8[.]0([ .-]|$)' <<<"$ffmpeg_version"
grep -Eq '^ffprobe version 8[.]0([ .-]|$)' <<<"$ffprobe_version"
grep -Eq ' vidstabdetect +V->V ' <<<"$filters"
grep -Eq ' vidstabtransform +V->V ' <<<"$filters"
printf '%s\n' "$BIN" >>"${GITHUB_PATH:?GITHUB_PATH is required}"

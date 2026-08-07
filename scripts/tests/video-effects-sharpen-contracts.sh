#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
for module in common pixels showcase-pixels showcase-effects; do
  # shellcheck source=/dev/null
  source "$ROOT/scripts/example-render-evidence/$module.sh"
done
tmp=$(mktemp -d "${TMPDIR:-/tmp}/veac-sharpen-contracts.XXXXXX")
trap 'rm -rf "$tmp"' EXIT
video="$tmp/no-sharpen.mp4"
ffmpeg -nostdin -v error -y -f lavfi \
  -i 'testsrc2=s=480x270:r=12:d=2' -c:v libx264 -pix_fmt yuv420p "$video"
log="$tmp/no-sharpen.log"
if (assert_sharpen_evidence "$video") >"$log" 2>&1; then
  fail 'no-sharpen negative fixture unexpectedly passed'
fi
rg -qF 'video-effects sharpen does not produce an edge halo' "$log" || {
  cat "$log" >&2
  fail 'no-sharpen fixture failed for the wrong reason'
}
printf 'video-effects sharpen contracts passed\n'

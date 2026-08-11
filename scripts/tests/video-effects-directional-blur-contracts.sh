#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
for module in common pixels showcase-pixels showcase-effects; do
  # shellcheck source=/dev/null
  source "$ROOT/scripts/example-render-evidence/$module.sh"
done
tmp=$(mktemp -d "${TMPDIR:-/tmp}/veac-directional-contracts.XXXXXX")
trap 'rm -rf "$tmp"' EXIT
plate="$ROOT/examples/video-effects/assets/effects-plate.png"

ffmpeg -nostdin -v error -y -loop 1 -framerate 4 -t 14 -i "$plate" \
  -filter_complex '
    [0:v]split=5[base][horizontal][vertical][early][peak];
    [base]trim=duration=10,setpts=PTS-STARTPTS[base-out];
    [horizontal]trim=duration=1,dblur=angle=0:radius=28,setpts=PTS-STARTPTS[h-out];
    [vertical]trim=duration=1,dblur=angle=90:radius=28,setpts=PTS-STARTPTS[v-out];
    [early]trim=duration=0.5,setpts=PTS-STARTPTS[early-out];
    [peak]trim=duration=1.5,dblur=angle=45:radius=36,setpts=PTS-STARTPTS[peak-out];
    [base-out][h-out][v-out][early-out][peak-out]concat=n=5:v=1:a=0,scale=160:90[out]
  ' -map '[out]' -c:v libx264 -pix_fmt yuv420p "$tmp/directional.mp4"
assert_directional_blur_evidence "$tmp/directional.mp4"

ffmpeg -nostdin -v error -y -loop 1 -framerate 4 -t 14 -i "$plate" \
  -vf 'scale=160:90' -c:v libx264 -pix_fmt yuv420p "$tmp/no-direction.mp4"
log="$tmp/no-direction.log"
if (assert_directional_blur_evidence "$tmp/no-direction.mp4") >"$log" 2>&1; then
  fail 'no-direction negative fixture unexpectedly passed'
fi
rg -qF 'video-effects directional blur axes are not distinct' "$log" || {
  cat "$log" >&2
  fail 'no-direction fixture failed for the wrong reason'
}
printf 'video-effects directional blur contracts passed\n'

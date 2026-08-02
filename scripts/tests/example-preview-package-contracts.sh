#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)
# shellcheck source=../example-preview-artifacts.sh
source "$ROOT/scripts/example-preview-artifacts.sh"
tmp=$(mktemp -d "${TMPDIR:-/tmp}/veac-preview-package-contracts.XXXXXX")
trap 'rm -rf "$tmp"' EXIT
rendered="$tmp/rendered"
package="$rendered/stream"
canonical="$tmp/canonical.json"

fail() {
  echo "example preview package contract failed: $*" >&2
  exit 1
}

expect_failure() {
  local label=$1 project=$2
  if verify_all_preview_deliverables "$project" "$rendered" >/dev/null 2>&1; then
    fail "$label was accepted"
  fi
}

mkdir -p "$package"
ffmpeg -v error -y -f lavfi -i 'testsrc2=size=64x64:rate=12:duration=1' \
  -an -c:v libx264 -pix_fmt yuv420p -g 6 -sc_threshold 0 -f hls \
  -hls_time 0.5 -hls_list_size 0 -hls_playlist_type vod \
  -hls_segment_filename "$package/segment-%03d.ts" "$package/media.m3u8"
cat > "$package/master.m3u8" <<'HLS'
#EXTM3U
#EXT-X-STREAM-INF:BANDWIDTH=300000,RESOLUTION=64x64
media.m3u8
HLS
cat > "$canonical" <<'JSON'
{"project":{"render_configs":[{"deliverables":[{"id":"dlv_stream","target":{"type":"package","name":"stream"},"kind":{"type":"adaptive_package","settings":{"type":"hls","settings":{}}}}]}]}}
JSON

verify_all_preview_deliverables "$canonical" "$rendered" ||
  fail "valid HLS package was rejected"

mv "$package" "$rendered/stream.saved"
expect_failure "missing package directory" "$canonical"
mv "$rendered/stream.saved" "$package"

mv "$package/master.m3u8" "$tmp/master.m3u8"
expect_failure "missing HLS entrypoint" "$canonical"
mv "$tmp/master.m3u8" "$package/master.m3u8"

jq '.project.render_configs[0].deliverables[0].target.type = "socket"' \
  "$canonical" > "$tmp/unsupported.json"
expect_failure "unsupported deliverable target" "$tmp/unsupported.json"
jq '.project.render_configs[0].deliverables[0].target.name = "../stream"' \
  "$canonical" > "$tmp/dangerous-target.json"
expect_failure "dangerous package target" "$tmp/dangerous-target.json"
jq '.project.render_configs[0].deliverables[0].kind.settings.type = "dash"' \
  "$canonical" > "$tmp/unsupported-package.json"
expect_failure "unsupported package kind" "$tmp/unsupported-package.json"

mv "$package" "$rendered/real-stream"
ln -s real-stream "$package"
expect_failure "symlink package directory" "$canonical"
rm "$package"
mv "$rendered/real-stream" "$package"

ln -s media.m3u8 "$package/alias.m3u8"
expect_failure "symlink package member" "$canonical"
rm "$package/alias.m3u8"
mkfifo "$package/pipe"
expect_failure "non-regular package member" "$canonical"
rm "$package/pipe"

cp "$package/master.m3u8" "$tmp/master.safe.m3u8"
cat > "$package/master.m3u8" <<'HLS'
#EXTM3U
#EXT-X-STREAM-INF:BANDWIDTH=300000
../outside.m3u8
HLS
expect_failure "unsafe HLS reference" "$canonical"
cp "$tmp/master.safe.m3u8" "$package/master.m3u8"

segments=("$package"/*.ts)
[[ -f "${segments[0]}" ]] || fail "HLS fixture has no media segment"
for segment in "${segments[@]}"; do
  printf 'not transport stream\n' > "$segment"
done
expect_failure "undecodable HLS package" "$canonical"

echo "Example preview package contract tests passed."

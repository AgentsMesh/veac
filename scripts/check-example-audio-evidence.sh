#!/usr/bin/env bash
set -euo pipefail

PREVIEW_ROOT=${1:-examples-preview}
export PREVIEW_ROOT
SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)

for module in common pixels showcase-pixels audio-metrics audio audio-all-features; do
  # shellcheck source=/dev/null
  source "$SCRIPT_DIR/example-render-evidence/$module.sh"
done

check_audio_evidence
echo "rendered example audio evidence is valid"

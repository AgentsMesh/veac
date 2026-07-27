#!/usr/bin/env bash
set -euo pipefail

docs=(
  docs/language-design/grammar.md
  docs/language-design/agent-authoring.md
  docs/language-design/semantic-kernel.md
  docs/language-design/mapping.md
)

reject() {
  local pattern="$1"
  local message="$2"
  local matches
  if matches=$(rg -n "$pattern" "${docs[@]}"); then
    printf 'language docs contract failed: %s\n%s\n' "$message" "$matches" >&2
    exit 1
  fi
}

require_in() {
  local path="$1"
  local pattern="$2"
  local message="$3"
  if ! rg -q "$pattern" "$path"; then
    printf 'language docs contract failed: %s (%s)\n' "$message" "$path" >&2
    exit 1
  fi
}

reject 'locator[[:space:]]+(file|url|provider)\b' 'legacy resource locator syntax'
reject 'output video .*from sequence' 'legacy video output syntax'
reject 'switch angle [^;{]+;' 'multicam switch without a time range'
reject 'optional `base_revision`' 'EditBatch base_revision is required'
reject '"(sequence_id|track_id)"' 'set_clip_enabled has unknown owner fields'

for path in docs/language-design/{agent-authoring,mapping}.md; do
  require_in "$path" '"operation_id"' 'EditBatch operation_id is undocumented'
  require_in "$path" '"atomic"' 'EditBatch atomic mode is undocumented'
done

for path in docs/language-design/{agent-authoring,semantic-kernel,mapping}.md; do
  require_in "$path" 'cosine' 'LUT1D interpolation set is incomplete'
  require_in "$path" 'pyramid' 'LUT3D interpolation set is incomplete'
done

printf 'language docs contract passed\n'

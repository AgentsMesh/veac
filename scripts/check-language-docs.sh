#!/usr/bin/env bash
set -euo pipefail

docs=(
  README.md
  docs/getting-started.md
  docs/language-reference/README.md
  docs/language-reference/outputs.md
  docs/language-reference/project.md
  docs/language-design/grammar.md
  docs/language-design/agent-authoring.md
  docs/language-design/semantic-kernel.md
  docs/language-design/mapping.md
  docs/rfcs/agent-authoring-and-canonical-ir.md
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
reject 'provider locators?' 'authoring supports only local and remote locators'
reject '(^|[[:space:]])output[[:space:]]+(video|image-sequence|caption-sidecar|audio-stem|scope|audio-file|animated-image|still-image|adaptive-package)\b' 'legacy authoring declaration'
reject '(^|[[:space:]])file-name[[:space:]]' 'legacy artifact path field'
reject 'encoding[[:space:]]*\{' 'anonymous artifact recipe block'
reject '(^|[^[:alnum:]_])([Vv]3|[Vv]4)([^[:alnum:]_]|$)|schema[- ]v[34]|schema version [34]' 'stale authoring or canonical project version'
reject '"(schema_version|min_reader_version)"[[:space:]]*:[[:space:]]*[34]' 'stale canonical project envelope'
reject '\+[[:space:]]+outputs\b|project[[:space:]`-]+outputs?\b' 'legacy project ownership terminology'
reject 'authoring output|output declarations?|output variants?|Outputs (are|form)' 'legacy authoring union terminology'
reject '(^|[[:space:]])format[[:space:]]+(mp3|gif|hls|png|jpeg|tiff|exr|srt|web-vtt|ass|wav|flac)\b' 'legacy recipe discriminator'
reject '(^|[[:space:]])loop-count[[:space:]]|(^|[[:space:]])variant[[:space:]]*\{' 'legacy GIF or HLS recipe'
reject '(video|audio)[[:space:]]*\{[[:space:]]*codec[[:space:]]' 'legacy generic codec block'
reject 'switch angle [^;{]+;' 'multicam switch without a time range'
reject 'optional `base_revision`' 'EditBatch base_revision is required'
reject '"(sequence_id|track_id)"' 'set_clip_enabled has unknown owner fields'

for path in docs/language-design/{agent-authoring,mapping}.md; do
  require_in "$path" '^```json,canonical-edit-batch$' 'EditBatch fence is not test-addressable'
  require_in "$path" '"operation_id"' 'EditBatch operation_id is undocumented'
  require_in "$path" '"atomic"' 'EditBatch atomic mode is undocumented'
done

for path in docs/language-design/{agent-authoring,semantic-kernel,mapping}.md; do
  require_in "$path" 'cosine' 'LUT1D interpolation set is incomplete'
  require_in "$path" 'pyramid' 'LUT3D interpolation set is incomplete'
done

require_in docs/language-design/grammar.md 'delivery \| annotation' 'delivery is missing from project ownership grammar'
require_in docs/language-design/mapping.md 'schema version 5' 'canonical project schema version is stale'
require_in docs/language-design/mapping.md 'minimum reader 5' 'canonical minimum reader is stale'
require_in docs/language-design/semantic-kernel.md 'schema version 5' 'semantic kernel schema version is stale'
require_in docs/language-design/agent-authoring.md 'layout standard;' 'agent video recipe hides mux layout'
require_in docs/language-design/agent-authoring.md 'passes single;' 'agent video recipe hides pass mode'
require_in docs/language-design/agent-authoring.md 'accelerator auto;' 'agent video recipe hides accelerator choice'
require_in docs/getting-started.md 'artifact video' 'getting started does not author a typed artifact'

delivery_doc=docs/language-reference/outputs.md
for kind in video image-sequence caption-sidecar audio-stem scope audio-file animated-image still-image adaptive-package; do
  require_in "$delivery_doc" "artifact $kind" "delivery reference omits $kind"
done
for primitive in 'mux mp4' 'encode mp3' 'encode gif' 'frame containing' 'package hls' 'rendition mobile'; do
  require_in "$delivery_doc" "$primitive" "delivery reference omits $primitive"
done

printf 'language docs contract passed\n'

#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)
cd "$ROOT"

current_docs=(
  README.md
  docs/getting-started.md
  docs/architecture.md
  docs/cli-reference.md
)
while IFS= read -r path; do current_docs+=("$path"); done < <(
  find docs/language-reference docs/language-design docs/rfcs \
    -type f -name '*.md' -print | LC_ALL=C sort
)

fail() {
  printf 'language docs contract failed: %s\n' "$1" >&2
  exit 1
}

for path in "${current_docs[@]}"; do
  [[ -f "$path" && ! -L "$path" ]] || fail "active document is not a regular file: $path"
done

reject() {
  local pattern="$1"
  local message="$2"
  local matches
  if matches=$(rg -n "$pattern" "${current_docs[@]}"); then
    printf 'language docs contract failed: %s\n%s\n' "$message" "$matches" >&2
    exit 1
  fi
}

reject_in() {
  local pattern="$1"
  local message="$2"
  shift 2
  local matches
  if matches=$(rg -n "$pattern" "$@"); then
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

require_linked() {
  local path="$1"
  shift
  local candidate name
  name=$(basename "$path")
  while IFS= read -r candidate; do
    [[ $candidate == "$path" ]] || return 0
  done < <(rg -Fl "$name" "$@" || true)
  fail "active document is not linked: $path"
}

for path in docs/language-reference/*.md; do
  [[ $path == docs/language-reference/README.md ]] && continue
  require_linked "$path" docs/language-reference/*.md
done
for path in docs/language-design/executable-stdlib-v6/*.md; do
  [[ $path == docs/language-design/executable-stdlib-v6/README.md ]] && continue
  require_linked "$path" docs/language-design/executable-stdlib-v6/README.md
done
for path in docs/rfcs/*.md; do
  require_linked "$path" README.md docs/*.md docs/language-*/*.md docs/rfcs/*.md
done

reject_in 'schema[- ]v[34567]|schema( version)? [34567]|minimum reader( version)? [34567]' \
  'stale canonical project version' README.md docs/architecture.md \
  docs/language-reference/README.md docs/language-design/mapping.md \
  docs/language-design/semantic-kernel.md docs/rfcs/agent-authoring-and-canonical-ir.md
reject 'typed authoring Document|core `Document`|authoring AST|authoring parser|second frontend' \
  'deleted legacy frontend terminology'
reject '(^|[[:space:]`])veac compile([[:space:]`]|$)|(^|[[:space:]`])compile main[.]veac' \
  'deleted compile command'
reject 'project[[:space:]]*\{[[:space:]]*(settings|resource|sequence|delivery)' \
  'deleted property-block project syntax'
reject '(^|[[:space:]])artifact[[:space:]]+(video|image-sequence|caption-sidecar|audio-stem|scope|audio-file|animated-image|still-image|adaptive-package)\b' \
  'deleted artifact declaration syntax'
reject '(^|[[:space:]])(preset|component|instance)[[:space:]]+(sequence|modifier|audio-processor)\b' \
  'deleted preset/component macro syntax'
reject 'locator[[:space:]]+(file|url|provider)\b|(^|[[:space:]])file-name[[:space:]]' \
  'deleted resource or artifact property syntax'

for path in docs/language-design/{agent-authoring,mapping}.md; do
  require_in "$path" '^```json,canonical-edit-batch$' 'EditBatch fence is not test-addressable'
  require_in "$path" '"operation_id"' 'EditBatch operation_id is undocumented'
  require_in "$path" '"base_revision"' 'EditBatch base_revision is undocumented'
  require_in "$path" '"atomic"' 'EditBatch atomic mode is undocumented'
done

for path in README.md docs/getting-started.md docs/language-design/agent-authoring.md; do
  require_in "$path" 'main\(Context\) -> Project' 'single executable entry ABI is undocumented'
  require_in "$path" 'animate' 'authored Temporal syntax is undocumented'
done

for path in docs/language-design/{mapping,semantic-kernel}.md; do
  require_in "$path" 'schema version 9' 'canonical project schema version is stale'
  require_in "$path" 'minimum reader 9|schema-v9' 'canonical minimum reader is stale'
done

require_in docs/language-design/semantic-kernel.md 'centered-only true overlaps' \
  'true-overlap transition contract is missing'
require_in docs/language-design/semantic-kernel.md 'held-frame `tpad`' \
  'transition endpoint padding prohibition is missing'
require_in docs/language-design/mapping.md 'SourceEditBatch' \
  'source-of-truth edit boundary is missing'
require_in docs/language-reference/source-editing.md 'source of truth|source-of-truth' \
  'source-of-truth editing is undocumented'
require_in docs/language-design/semantic-kernel.md 'cosine' 'LUT1D interpolation set is incomplete'
require_in docs/language-design/semantic-kernel.md 'pyramid' 'LUT3D interpolation set is incomplete'

delivery_doc=docs/language-reference/outputs.md
for constructor in deliverable_video deliverable_image_frames deliverable_caption_sidecar \
  deliverable_audio_stem deliverable_scope deliverable_mp3 deliverable_gif \
  deliverable_still deliverable_hls; do
  require_in "$delivery_doc" "$constructor" "delivery reference omits $constructor"
done

printf 'language docs contract passed\n'

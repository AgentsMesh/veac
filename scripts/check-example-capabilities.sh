#!/usr/bin/env bash
set -euo pipefail

ROOT="${VEAC_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
CATALOG="${1:-$ROOT/examples/capabilities.json}"
MATRIX="$ROOT/docs/capability-matrix-roadmap.md"
GALLERY_SCHEMA="$ROOT/scripts/check-gallery-catalog.jq"

fail() {
  echo "example capability check failed: $*" >&2
  exit 1
}

command -v jq >/dev/null 2>&1 || fail "jq is required"
[[ -f "$CATALOG" ]] || fail "missing catalog: $CATALOG"
[[ -f "$MATRIX" ]] || fail "missing capability matrix: $MATRIX"
[[ -f "$GALLERY_SCHEMA" ]] || fail "missing gallery schema: $GALLERY_SCHEMA"

jq -e '
  def nonempty: type == "string" and length > 0;
  .schema_version == 1 and
  (.capabilities | type == "array" and length > 0) and
  (.gallery_catalog | nonempty) and
  (.mechanism_catalogs | type == "array" and length > 0 and all(.[]; nonempty)) and
  all(.capabilities[];
    . as $entry |
    ($entry.id | nonempty and test("^(P1|P2)-[0-9]{2}$")) and
    ($entry.layer | nonempty) and
    ($entry.layer as $layer | ["ir", "dsl", "edit", "plan", "codegen",
      "runtime", "execution", "cli", "interchange", "provider"] |
      index($layer) != null) and
    ($entry.status == "stable" or $entry.status == "contract") and
    ($entry | has("example")) and
    ($entry.evidence | type == "array" and length > 0 and all(.[]; nonempty)) and
    ([$entry.example, ($entry.not_applicable // null)] |
      map(select(nonempty)) | length == 1) and
    (if ($entry.not_applicable | type == "string") then
      ($entry.not_applicable | nonempty and test("[一-龥]"))
    else true end) and
    (if $entry.layer == "dsl" and $entry.status == "stable" then
      (($entry.example | nonempty) or ($entry.not_applicable | nonempty))
    else true end)
  )
' "$CATALOG" >/dev/null || fail "catalog schema or coverage declaration is invalid"

duplicate_ids="$(jq -r '.capabilities[].id' "$CATALOG" | sort | uniq -d)"
[[ -z "$duplicate_ids" ]] || fail "duplicate capability IDs: $duplicate_ids"

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

gallery_rel="$(jq -r '.gallery_catalog' "$CATALOG")"
[[ "$gallery_rel" != /* && "$gallery_rel" != *".."* ]] ||
  fail "gallery catalog path must be repository-relative"
gallery="$ROOT/$gallery_rel"
[[ -f "$gallery" && ! -L "$gallery" ]] || fail "missing gallery catalog: $gallery_rel"
jq -e -f "$GALLERY_SCHEMA" "$gallery" >/dev/null ||
  fail "gallery catalog schema is invalid"

: > "$tmp/mechanisms.jsonl"
while IFS= read -r fragment_rel; do
  [[ "$fragment_rel" != /* && "$fragment_rel" != *".."* ]] ||
    fail "mechanism catalog path must be repository-relative: $fragment_rel"
  fragment="$ROOT/$fragment_rel"
  [[ -f "$fragment" ]] || fail "missing mechanism catalog: $fragment_rel"
  jq -e '.schema_version == 1 and (.mechanisms | type == "array" and length > 0)' \
    "$fragment" >/dev/null || fail "invalid mechanism fragment: $fragment_rel"
  jq -c '.mechanisms[]' "$fragment" >> "$tmp/mechanisms.jsonl"
done < <(jq -r '.mechanism_catalogs[]' "$CATALOG")
jq -s '.' "$tmp/mechanisms.jsonl" > "$tmp/mechanisms.json"

jq -e --slurpfile gallery "$gallery" '
  def nonempty: type == "string" and length > 0;
  def count_prefix($prefix): map(select(.id | startswith($prefix))) | length;
  all(.[];
    . as $mechanism |
    ($mechanism.id | nonempty and test("^[a-z][a-z0-9.-]+$")) and
    ($mechanism.title | nonempty and test("[一-龥]")) and
    ($mechanism.family | nonempty and test("[一-龥]")) and
    ($mechanism.coverage as $coverage |
      ["preview_required", "workflow_evidence", "external"] |
      index($coverage) != null) and
    ($mechanism | has("gallery_target") or has("capability_ids") | not) and
    (if $mechanism.coverage == "preview_required" then
      ($mechanism.preview_target | nonempty) and
      ($mechanism.capability_id | nonempty and test("^(P1|P2)-[0-9]{2}$")) and
      ($mechanism | has("evidence") | not) and
      any($gallery[0].targets[];
        .id == $mechanism.preview_target and .kind == "render" and
        (.example | nonempty) and .preview_window != null and
        (.capability_ids | index($mechanism.capability_id)) != null)
    else
      ($mechanism.evidence | nonempty) and
      ($mechanism | has("preview_target") or has("capability_id") | not)
    end)
  ) and
  count_prefix("effect.") >= 10 and count_prefix("transition.") >= 7 and
  count_prefix("blend.") >= 12 and
  (count_prefix("mask.") + count_prefix("matte.")) >= 8 and
  count_prefix("source-time.") >= 6 and count_prefix("audio.") >= 10 and
  count_prefix("text.") >= 12 and count_prefix("color.") >= 8 and
  count_prefix("delivery.") >= 5
' "$tmp/mechanisms.json" >/dev/null || fail "granular mechanism declaration is invalid"

awk -F'|' '
  function trim(value) {
    gsub(/^[[:space:]]+|[[:space:]]+$/, "", value)
    return value
  }
  {
    id = trim($2)
    gsub(/`/, "", id)
    if (id !~ /^P[12]-[0-9][0-9]$/) next
    state = trim($5)
    gsub(/`/, "", state)
    if (state == "Delivered") state = "stable"
    else if (state == "Delivered/Contract" || state == "Delivered \/ Contract") state = "contract"
    else state = "invalid:" state
    print id "\t" state
  }
' "$MATRIX" | sort > "$tmp/matrix.tsv"

jq -r '.capabilities[] | [.id, .status] | @tsv' "$CATALOG" |
  sort > "$tmp/catalog.tsv"

if ! diff -u "$tmp/matrix.tsv" "$tmp/catalog.tsv" > "$tmp/diff"; then
  cat "$tmp/diff" >&2
  fail "catalog IDs or statuses differ from the capability matrix"
fi

jq -r '.capabilities[].id' "$CATALOG" | sort -u > "$tmp/capability-ids"
{
  jq -r '.targets[].capability_ids[]' "$gallery"
  jq -r '.[] | .capability_id // empty' "$tmp/mechanisms.json"
} | sort -u > "$tmp/referenced-capability-ids"
unknown="$(comm -23 "$tmp/referenced-capability-ids" "$tmp/capability-ids")"
[[ -z "$unknown" ]] || fail "unknown referenced capability IDs: $unknown"

jq -r '.targets[].id' "$gallery" | sort -u > "$tmp/gallery-targets"
duplicate_targets="$(jq -r '.targets[].id' "$gallery" | sort | uniq -d)"
[[ -z "$duplicate_targets" ]] || fail "duplicate gallery targets: $duplicate_targets"
duplicate_mechanisms="$(jq -r '.[].id' "$tmp/mechanisms.json" | sort | uniq -d)"
[[ -z "$duplicate_mechanisms" ]] || fail "duplicate mechanism IDs: $duplicate_mechanisms"
unknown_targets="$(jq -r '.[] | .preview_target // empty' "$tmp/mechanisms.json" | sort -u |
  comm -23 - "$tmp/gallery-targets")"
[[ -z "$unknown_targets" ]] || fail "unknown gallery targets: $unknown_targets"

while IFS= read -r example; do
  [[ "$example" == examples/*.veac ]] || fail "invalid example path: $example"
  [[ -f "$ROOT/$example" ]] || fail "missing example: $example"
done < <({
  jq -r '.capabilities[].example // empty' "$CATALOG"
  jq -r '.targets[].example // empty' "$gallery"
} | sort -u)

while IFS= read -r evidence; do
  [[ "$evidence" != /* && "$evidence" != *".."* ]] ||
    fail "evidence path must be repository-relative: $evidence"
  [[ -f "$ROOT/$evidence" ]] || fail "missing evidence: $evidence"
done < <({
  jq -r '.capabilities[].evidence[]' "$CATALOG"
  jq -r '.targets[].workflow_evidence[]?' "$gallery"
  jq -r '.[] | .evidence // empty' "$tmp/mechanisms.json"
} | sort -u)

"$ROOT/scripts/check-example-registry.sh" "$tmp/mechanisms.json" >/dev/null

echo "Example capability catalog is valid."

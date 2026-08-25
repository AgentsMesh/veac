#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)
source "$ROOT/scripts/example-preview-entry.sh"
TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT
GALLERY="$TMP/gallery.json"
PREVIEW_FILTER="$ROOT/scripts/example-preview.jq"
PREVIEW_EDGE=480
PREVIEW_FPS=12
VEAC_FAKE_LOG="$TMP/veac.log"
VEAC_FAKE_CANONICAL="$TMP/canonical.json"
export VEAC_FAKE_LOG VEAC_FAKE_CANONICAL

fail() {
  echo "preview build flow contract failed: $*" >&2
  return 1
}

materialize_preview_assets() { :; }
pad_preview_audio() { :; }
verify_expected_preview_artifacts() {
  [[ $2 == */project/project.veac.json ]] || return 1
  [[ $3 == */project/project.preview.veac.json ]] || return 1
  [[ $4 == */plans/preview/out_4a4f4ce03f87.json ]] || return 1
  [[ $5 == */rendered ]] || return 1
}
run_example_preview_process() {
  local project=$1
  shift
  if [[ ${MUTATE_AUTHORING:-0} == 1 ]]; then
    printf ' ' >>"$(dirname "$project")/project/project.veac.json"
  fi
  if [[ ${MUTATE_SOURCE:-0} == 1 ]]; then
    printf ' ' >>"$(dirname "$project")/project/main.veac"
  fi
  if [[ ${MUTATE_MODULE:-0} == 1 ]]; then
    printf ' ' >>"$(dirname "$project")/project/brand.veac"
  fi
  if [[ ${MUTATE_PREVIEW:-0} == 1 ]]; then
    printf ' ' >>"$(dirname "$project")/project/project.preview.veac.json"
  fi
  if [[ ${MUTATE_SOURCE_EDIT:-0} == 1 ]]; then
    printf ' ' >>"$(dirname "$project")/project/source-edit.outcome.json"
  fi
  (cd "$project" && "$@")
}

cat >"$GALLERY" <<'JSON'
{"targets":[{"id":"demo","frontend":"executable","preview_window":null,"expected_artifacts":[
  {"kind":"canonical_project"},{"kind":"preview_canonical_project"},
  {"kind":"preview_resolved_plan"},
  {"kind":"source_revision"},{"kind":"source_index"},
  {"kind":"source_edit_batch"},{"kind":"source_edit_outcome"},
  {"kind":"delivery","logical_key":"preview","artifacts":[
    {"kind":"video","target_type":"file","target":"preview.mp4"}]}]}]}
JSON
cat >"$VEAC_FAKE_CANONICAL" <<'JSON'
{"project":{"id":"prj_demo","materials":[],"authorship":{"entity":{"logical_path":["demo"],"events":[]},"multicam_groups":[],"annotations":[],"deliveries":[{"render_config_id":"out_4a4f4ce03f87","entity":{"logical_path":["demo","preview"],"events":[]}}]},"sequences":[{"id":"seq_main","settings":{"frame_rate":{"numerator":30,"denominator":1}},"tracks":[],"applies":[]}],"relations":[],"annotations":[{"style":{"font":{"type":"family","family":"Arial"},"fallback_fonts":[{"type":"family","family":"Noto Sans Arabic"}]}}],"render_configs":[{"id":"out_4a4f4ce03f87","sequence_id":"seq_main","raster":{"width":1280,"height":720,"frame_rate":{"numerator":30,"denominator":1}},"deliverables":[{"id":"dlv_92f3","target":{"type":"file","name":"preview.mp4"},"kind":{"type":"video","settings":{"hardware":{"type":"auto"}}}}]}]}}
JSON
mkdir -p "$TMP/source/demo"
cat >"$TMP/source/demo/main.veac" <<'VEAC'
language "veac" version 6;
import "./brand.veac" as brand;
fn main(context: Context) -> Project {
  let duration = 1s;
  project(identifier("demo"), project_settings(1))
}
VEAC
cat >"$TMP/source/demo/brand.veac" <<'VEAC'
language "veac" version 6;
pub let duration: Time = 1s;
VEAC
cat >"$TMP/source/demo/source-edit.json" <<'JSON'
{"schema":"https://veac.dev/schemas/source-edit","schema_version":9,
"operation_id":"op_demo_source_edit",
"base_revision":{"authored_source_graph_sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","complete_source_graph_sha256":"dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"},
"atomic":true,"preconditions":[],"operations":[
{"type":"insert_import","module":"main.veac","anchor":{"type":"after_import",
"target":{"module":"main.veac","alias":"brand"}},
"import":{"path":"./brand.veac","alias":"brand"}},
{"type":"remove_import","target":{"module":"main.veac","alias":"brand"}},
{"type":"insert_declaration","module":"brand.veac","anchor":{"type":"after_declaration",
"target":{"module":"brand.veac","path":{"kind":"constant","constant":"duration"}}},
"declaration":{"source":"pub let replacement: Time = 2s;"}},
{"type":"remove_declaration","target":{"module":"brand.veac",
"path":{"kind":"constant","constant":"duration"}}},
{"type":"set_statement","target":{"module":"main.veac",
"path":{"kind":"function","function":"main"}},"site":{"type":"body_statement",
"path":{"steps":[{"step":"local_value","operation":"let","binding":"duration",
"ordinal":0}]}},"statement":{"source":"let duration = 2s;"}}]}
JSON
cat >"$TMP/fake-veac" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$*" >>"$VEAC_FAKE_LOG"
case $1 in
  source-revision)
    printf '%s\n' '{"authored_source_graph_sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","complete_source_graph_sha256":"dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"}' ;;
  source-index)
    printf '%s\n' '{"schema":"https://veac.dev/schemas/source-index","schema_version":11,"revision":{"authored_source_graph_sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","complete_source_graph_sha256":"dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"},"build_inputs":[],"modules":[{"module":"brand.veac","range":{"start":0,"end":40},"imports":[],"declarations":[{"target":{"module":"brand.veac","path":{"kind":"constant","constant":"duration"}},"source":"pub let duration: Time = 1s;","range":{"start":0,"end":29}}]},{"module":"main.veac","range":{"start":0,"end":140},"imports":[{"target":{"module":"main.veac","alias":"brand"},"path":"./brand.veac","source":"import \"./brand.veac\" as brand;","range":{"start":0,"end":31}}],"declarations":[]}],"nodes":[{"target":{"module":"brand.veac","path":{"kind":"constant","constant":"duration"}},"range":{"start":0,"end":29},"expressions":[{"site":{"type":"constant_value"},"source":"1s","range":{"start":26,"end":28}}],"statements":[],"bodies":[],"declarations":[]},{"target":{"module":"main.veac","path":{"kind":"function","function":"main"}},"range":{"start":60,"end":140},"expressions":[],"statements":[{"site":{"type":"body_statement","path":{"steps":[{"step":"local_value","operation":"let","binding":"duration","ordinal":0}]}},"source":"let duration = 1s;","range":{"start":90,"end":109}}],"bodies":[],"declarations":[]}]}' ;;
  source-edit)
    : >"$(dirname "$2")/.veac-source.lock"
    printf '%s\n' '{"modules":["brand.veac","main.veac"],"previous_revision":{"authored_source_graph_sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","complete_source_graph_sha256":"dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"},"new_revision":{"authored_source_graph_sha256":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","complete_source_graph_sha256":"eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"},"destinations":[],"dry_run":true}' ;;
  build) cp "$VEAC_FAKE_CANONICAL" "$3" ;;
  plan)
    printf '{"output":{"render_config_id":"%s"}}\n' "$3" ;;
  render)
    destination=''; previous=''
    for argument in "$@"; do
      [[ $previous == --destination ]] && destination=$argument
      previous=$argument
    done
    mkdir -p "$destination"
    printf 'fake media\n' >"$destination/preview.mp4" ;;
  *) exit 2 ;;
esac
SH
chmod +x "$TMP/fake-veac"

mkdir -p "$TMP/output"
build_example "$TMP/source/demo" "$TMP/output" "$TMP/fixtures" "$TMP/fake-veac"
ENTRY="$TMP/output/demo"
cmp -s "$TMP/source/demo/main.veac" "$ENTRY/project/main.veac" ||
  fail "authoring source bytes changed"
cmp -s "$TMP/source/demo/brand.veac" "$ENTRY/project/brand.veac" ||
  fail "imported module bytes changed"
cmp -s "$TMP/source/demo/source-edit.json" "$ENTRY/project/source-edit.json" ||
  fail "source edit batch bytes changed"
[[ -f $ENTRY/project/.veac-source.lock && ! -L $ENTRY/project/.veac-source.lock ]] ||
  fail "source edit lock contract drifted"
[[ ! -e $ENTRY/project/main.preview.veac ]] || fail "derived preview source was published"
jq -e '.project.render_configs[0].raster.width == 1280 and
  .project.render_configs[0].raster.frame_rate.numerator == 30' \
  "$ENTRY/project/project.veac.json" >/dev/null || fail "authoring canonical was transformed"
jq -e '.project.render_configs[0].raster == {"width":480,"height":270,
  "frame_rate":{"numerator":12,"denominator":1}}' \
  "$ENTRY/project/project.preview.veac.json" >/dev/null || fail "preview policy was not applied"
jq -e '.project.annotations[0].style.font.material_id == "med_preview-font" and
  .project.annotations[0].style.fallback_fonts[0].material_id ==
    "med_preview-arabic-font" and (.project.materials | length) == 2' \
  "$ENTRY/project/project.preview.veac.json" >/dev/null ||
  fail "expanded source graph fonts were not adapted in preview IR"
[[ $(rg -c '^build ' "$VEAC_FAKE_LOG") == 1 ]] || fail "source graph was not built once"
rg -q '^source-revision .*main.veac$' "$VEAC_FAKE_LOG" || fail "source revision command drifted"
rg -q '^source-index .*main.veac$' "$VEAC_FAKE_LOG" || fail "source index command drifted"
rg -q '^source-edit .*source-edit.json --dry-run$' "$VEAC_FAKE_LOG" ||
  fail "source edit command drifted"
rg -q "^plan .*project.preview.veac.json$" "$VEAC_FAKE_LOG" ||
  fail "plan did not consume preview canonical"
rg -q "^render .*project.preview.veac.json$" "$VEAC_FAKE_LOG" ||
  fail "render did not consume preview canonical"
reject_legacy_preview_layout "$ENTRY"

cp "$ENTRY/project/source-edit.outcome.json" "$TMP/source-edit.outcome.json"
jq '.dry_run = false' "$TMP/source-edit.outcome.json" \
  >"$ENTRY/project/source-edit.outcome.json"
if verify_example_source_edit_evidence "$ENTRY" >/dev/null 2>&1; then
  fail "non-dry-run source edit outcome was accepted"
fi
cp "$TMP/source-edit.outcome.json" "$ENTRY/project/source-edit.outcome.json"
cp "$ENTRY/project/source-edit.json" "$TMP/source-edit.json"
jq '.base_revision.complete_source_graph_sha256 = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"' \
  "$TMP/source-edit.json" >"$ENTRY/project/source-edit.json"
if verify_example_source_edit_evidence "$ENTRY" >/dev/null 2>&1; then
  fail "stale source edit batch was accepted"
fi
cp "$TMP/source-edit.json" "$ENTRY/project/source-edit.json"
mv "$ENTRY/project/source.index.json" "$TMP/source.index.json"
if verify_example_source_edit_evidence "$ENTRY" >/dev/null 2>&1; then
  fail "missing source index was accepted"
fi
mv "$TMP/source.index.json" "$ENTRY/project/source.index.json"

for mutation in MUTATE_AUTHORING MUTATE_SOURCE MUTATE_MODULE MUTATE_PREVIEW MUTATE_SOURCE_EDIT; do
  unset MUTATE_AUTHORING MUTATE_SOURCE MUTATE_MODULE MUTATE_PREVIEW MUTATE_SOURCE_EDIT
  printf -v "$mutation" '%s' 1
  mkdir -p "$TMP/output-$mutation"
  if build_example "$TMP/source/demo" "$TMP/output-$mutation" "$TMP/fixtures" \
      "$TMP/fake-veac" >/dev/null 2>&1; then
    fail "$mutation was accepted"
  fi
done

echo "example preview build flow contracts passed"

#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)
source "$ROOT/scripts/example-preview-entry.sh"
TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT
GALLERY="$TMP/gallery.json"
PREPARE_SOURCE="$ROOT/scripts/prepare-example-source.sh"
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
  [[ $4 == */plans/preview/out_preview.json ]] || return 1
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
  if [[ ${MUTATE_PREVIEW:-0} == 1 ]]; then
    printf ' ' >>"$(dirname "$project")/project/project.preview.veac.json"
  fi
  (cd "$project" && "$@")
}

cat >"$GALLERY" <<'JSON'
{"targets":[{"id":"demo","preview_window":null,"expected_artifacts":[
  {"kind":"canonical_project"},{"kind":"preview_canonical_project"},
  {"kind":"preview_resolved_plan"},
  {"kind":"authoring_delivery","id":"preview","artifact_ids":["preview"]}]}]}
JSON
cat >"$VEAC_FAKE_CANONICAL" <<'JSON'
{"project":{"id":"prj_demo","materials":[],"sequences":[{"id":"seq_main","settings":{"frame_rate":{"numerator":30,"denominator":1}},"tracks":[],"applies":[]}],"relations":[],"annotations":[],"render_configs":[{"id":"out_preview","sequence_id":"seq_main","raster":{"width":1280,"height":720,"frame_rate":{"numerator":30,"denominator":1}},"deliverables":[{"id":"dlv_preview","target":{"type":"file","name":"preview.mp4"},"kind":{"type":"video","settings":{"hardware":{"type":"auto"}}}}]}]}}
JSON
mkdir -p "$TMP/source/demo"
cat >"$TMP/source/demo/main.veac" <<'VEAC'
project demo {
  font family "Arial";
  sequence main {}
}
VEAC
cat >"$TMP/fake-veac" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$*" >>"$VEAC_FAKE_LOG"
case $1 in
  compile) cp "$VEAC_FAKE_CANONICAL" "$3" ;;
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
rg -q 'font resource preview-font' "$ENTRY/project/main.preview.veac" ||
  fail "preview source was not adapted"
jq -e '.project.render_configs[0].raster.width == 1280 and
  .project.render_configs[0].raster.frame_rate.numerator == 30' \
  "$ENTRY/project/project.veac.json" >/dev/null || fail "authoring canonical was transformed"
jq -e '.project.render_configs[0].raster == {"width":480,"height":270,
  "frame_rate":{"numerator":12,"denominator":1}}' \
  "$ENTRY/project/project.preview.veac.json" >/dev/null || fail "preview policy was not applied"
[[ $(rg -c '^compile ' "$VEAC_FAKE_LOG") == 2 ]] || fail "expected two explicit compiles"
rg -q "^plan .*project.preview.veac.json$" "$VEAC_FAKE_LOG" ||
  fail "plan did not consume preview canonical"
rg -q "^render .*project.preview.veac.json$" "$VEAC_FAKE_LOG" ||
  fail "render did not consume preview canonical"
reject_legacy_preview_layout "$ENTRY"

for mutation in MUTATE_AUTHORING MUTATE_SOURCE MUTATE_PREVIEW; do
  unset MUTATE_AUTHORING MUTATE_SOURCE MUTATE_PREVIEW
  printf -v "$mutation" '%s' 1
  mkdir -p "$TMP/output-$mutation"
  if build_example "$TMP/source/demo" "$TMP/output-$mutation" "$TMP/fixtures" \
      "$TMP/fake-veac" >/dev/null 2>&1; then
    fail "$mutation was accepted"
  fi
done

echo "example preview build flow contracts passed"

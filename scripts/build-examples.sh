#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)
source "$ROOT/scripts/example-preview-fixtures.sh"
source "$ROOT/scripts/example-preview-lock.sh"
source "$ROOT/scripts/example-preview-process.sh"
source "$ROOT/scripts/example-preview-artifacts.sh"
source "$ROOT/scripts/example-preview-publish.sh"
GALLERY="$ROOT/examples/catalog/gallery.json"
GALLERY_SCHEMA="$ROOT/scripts/check-gallery-catalog.jq"
PREPARE_SOURCE="$ROOT/scripts/prepare-example-source.sh"
PREVIEW_FILTER="$ROOT/scripts/example-preview.jq"
ACTION=${1:-build}
OUTPUT_INPUT=${2:-"$ROOT/examples-preview"}
ONLY_EXAMPLES=${VEAC_EXAMPLES:-}
PREVIEW_EDGE=${VEAC_PREVIEW_MAX_EDGE:-240}
PREVIEW_FPS=${VEAC_PREVIEW_FPS:-12}
TOOLCHAIN=${RUSTUP_TOOLCHAIN:-1.85.0}
fail() {
  echo "examples preview: $*" >&2
  exit 1
}
cleanup() {
  local status=$?
  local cleanup_status=0
  trap - EXIT
  set +e
  stop_example_preview_process
  cleanup_preview_transaction "$ROOT" "${OUTPUT:-}" || cleanup_status=1
  release_example_preview_lock
  if [[ $status -eq 0 && $cleanup_status -ne 0 ]]; then status=1; fi
  exit "$status"
}
require_command() {
  command -v "$1" >/dev/null 2>&1 || fail "required command not found: $1"
}
normalize_output() {
  local input=$1
  local parent name
  [[ "$input" == /* ]] || input="$ROOT/$input"
  case "$input" in
    */../*|*/..|*/./*|*/.) fail "preview directory must be normalized: $input" ;;
  esac
  case "$input" in
    "$ROOT/examples-preview"|"$ROOT/examples-preview/"*) ;;
    *) fail "preview directory must stay under $ROOT/examples-preview" ;;
  esac
  name=$(basename "$input")
  [[ -n "$name" && "$name" != "." && "$name" != ".." ]] || \
    fail "invalid preview directory: $input"
  mkdir -p "$(dirname "$input")"
  parent=$(cd "$(dirname "$input")" && pwd -P)
  printf '%s/%s\n' "$parent" "$name"
}
safe_remove_output() {
  local output=$1
  local relative
  case "$output" in
    "$ROOT/examples-preview"|"$ROOT/examples-preview/"*) ;;
    *) fail "refusing to remove non-preview directory: $output" ;;
  esac
  [[ ! -L "$output" ]] || fail "preview directory must not be a symlink: $output"
  relative=${output#"$ROOT"/}
  git -C "$ROOT" check-ignore -q -- "$relative/" || \
    fail "preview directory must be covered by .gitignore: $output"
  rm -rf "$output"
}

build_example() {
  local source_dir=$1
  local output=$2
  local fixtures=$3
  local veac=$4
  local name entry project source raw canonical plan rendered log config config_plan primary_config
  local target window
  name=$(basename "$source_dir")
  [[ "$name" =~ ^[a-z0-9][a-z0-9-]*$ ]] || fail "unsafe example name: $name"
  target=$(jq -ce --arg id "$name" '.targets[] | select(.id == $id)' "$GALLERY")
  [[ -n "$target" ]] || fail "gallery target missing for example: $name"
  window=$(jq -c '.preview_window' <<<"$target")
  entry="$output/$name"
  project="$entry/project"
  source="$project/main.veac"
  raw="$project/project.raw.json"
  canonical="$project/project.veac.json"
  plan="$entry/plan.json"
  rendered="$entry/rendered"
  log="$entry/build.log"
  mkdir -p "$project" "$rendered"
  [[ -f "$source_dir/main.veac" && ! -L "$source_dir/main.veac" ]] || \
    fail "example source must be a regular non-symlink file: $name"
  cp "$source_dir/main.veac" "$source"
  "$PREPARE_SOURCE" "$source"
  echo "[$name] compile -> plan -> render -> probe"
  if ! (
    "$veac" compile --emit-ir "$raw" "$source" || exit 1
    jq --argjson edge "$PREVIEW_EDGE" --argjson fps "$PREVIEW_FPS" \
      --argjson window "$window" \
      -f "$PREVIEW_FILTER" "$raw" > "$canonical" || exit 1
    materialize_preview_assets "$project" "$canonical" "$fixtures" || exit 1
    pad_preview_audio "$project" "$canonical" 65 || exit 1
    rm "$raw"
    primary_config=$(jq -r '
      ([.project.render_configs[] |
        select(any(.deliverables[]; .kind.type == "video")) |
        .id][0]) // .project.render_configs[0].id // empty
    ' "$canonical") || exit 1
    [[ -n "$primary_config" ]] || exit 1
    while IFS= read -r config; do
      config_plan="$entry/plan.$config.json"
      (cd "$project" && "$veac" plan --config "$config" "$canonical") > "$config_plan" || exit 1
      run_example_preview_process "$project" "$veac" render \
        --config "$config" --destination "$rendered" "$canonical" || exit 1
      if [[ "$config" == "$primary_config" ]]; then
        cp "$config_plan" "$plan" || exit 1
      fi
    done < <(jq -r '.project.render_configs[].id' "$canonical")
    rm -f "$rendered/.veac-render.lock"
    verify_expected_preview_artifacts "$target" "$canonical" "$plan" "$rendered" || exit 1
  ) > "$log" 2>&1; then
    cat "$log" >&2
    return 1
  fi
}

OUTPUT=$(normalize_output "$OUTPUT_INPUT")
acquire_example_preview_lock "$ROOT"
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
case "$ACTION" in
  clean)
    safe_remove_output "$OUTPUT"
    exit 0 ;;
  build) ;;
  *) fail "usage: $0 {build|clean} [preview-directory]" ;;
esac

for command in cargo ffmpeg ffprobe jq pgrep rg; do require_command "$command"; done
[[ -f "$GALLERY" && ! -L "$GALLERY" ]] || fail "gallery catalog must be a regular file"
jq -e -f "$GALLERY_SCHEMA" "$GALLERY" >/dev/null || fail "invalid gallery catalog"
[[ "$PREVIEW_EDGE" =~ ^[1-9][0-9]*$ ]] || fail "preview edge must be a positive integer"
[[ "$PREVIEW_FPS" =~ ^[1-9][0-9]*$ ]] || fail "preview FPS must be a positive integer"
TARGET_DIR=$(cargo +"$TOOLCHAIN" metadata --format-version 1 --no-deps | jq -r '.target_directory')
VEAC=${VEAC_BIN:-"$TARGET_DIR/debug/veac"}
[[ -x "$VEAC" ]] || fail "VEAC CLI not built: $VEAC"

SELECTOR=${ONLY_EXAMPLES//,/ }
EXAMPLES=()
AVAILABLE=()
while IFS= read -r source; do
  name=$(basename "$(dirname "$source")")
  AVAILABLE+=("$name")
  if [[ -z "$SELECTOR" || " $SELECTOR " == *" $name "* ]]; then
    EXAMPLES+=("$(dirname "$source")")
  fi
done < <(
  jq -r '.targets[] | select(.example != null) | .example' "$GALLERY" |
    sort -u |
    sed "s#^#$ROOT/#"
)
for requested in $SELECTOR; do
  [[ "$requested" =~ ^[a-z0-9][a-z0-9-]*$ ]] || fail "invalid example selector: $requested"
  [[ " ${AVAILABLE[*]} " == *" $requested "* ]] || fail "unknown example: $requested"
done
[[ ${#EXAMPLES[@]} -gt 0 ]] || fail "no examples found"
create_preview_staging "$ROOT" "$OUTPUT"
BUILD_OUTPUT=$PREVIEW_STAGING
FIXTURES="$BUILD_OUTPUT/.fixtures"
prepare_preview_fixtures "$FIXTURES"
for source_dir in "${EXAMPLES[@]}"; do
  build_example "$source_dir" "$BUILD_OUTPUT" "$FIXTURES" "$VEAC"
done
rm -rf "$FIXTURES"
bash "$ROOT/scripts/write-examples-index.sh" "$BUILD_OUTPUT" "${#EXAMPLES[@]}"
publish_preview_staging "$ROOT" "$OUTPUT"
echo "example previews: $OUTPUT/index.html"

#!/usr/bin/env bash

write_smoke_ebml_header() {
  local file=$1 doctype=$2
  case $doctype in
    matroska) printf '\032\105\337\243\213\102\202\210matroska' >"$file" ;;
    webm) printf '\032\105\337\243\207\102\202\204webm' >"$file" ;;
  esac
}

assert_smoke_ebml_fixture() (
  local file=$1 expected=$2 actual
  # shellcheck disable=SC2329
  ffprobe() { printf '%s\n' '{"format":{"format_name":"matroska,webm"}}'; }
  actual=$(smoke_ebml_doctype "$file")
  [[ $actual == "$expected" ]] || fail "wrong EBML DocType: $actual"
  assert_smoke_container "$file" "${expected/matroska/mkv}" "EBML $expected"
)

expect_smoke_ebml_mismatch() {
  local file=$1 expected=$2
  # shellcheck disable=SC2329
  if (ffprobe() { printf '%s\n' '{"format":{"format_name":"matroska,webm"}}'; }
      assert_smoke_container "$file" "$expected" "EBML mismatch") >/dev/null 2>&1; then
    fail "EBML $expected mismatch passed"
  fi
}

assert_smoke_interval_fixture() (
  local payload=$1
  # shellcheck disable=SC2329
  ffprobe() { printf '%s\n' "$payload"; }
  assert_smoke_video_interval ignored 1 12/1 "interval fixture"
)

expect_smoke_interval_failure() {
  local label=$1 payload=$2
  if assert_smoke_interval_fixture "$payload" >/dev/null 2>&1; then
    fail "$label video interval passed"
  fi
}

check_smoke_ebml_contracts() {
  local root=$1 matroska="$1/matroska.header" webm="$1/webm.header"
  write_smoke_ebml_header "$matroska" matroska
  write_smoke_ebml_header "$webm" webm
  assert_smoke_ebml_fixture "$matroska" matroska
  assert_smoke_ebml_fixture "$webm" webm
  expect_smoke_ebml_mismatch "$matroska" webm
  expect_smoke_ebml_mismatch "$webm" mkv
}

check_smoke_interval_contracts() {
  local valid negative positive short
  valid='{"streams":[{"start_time":"0"}],"packets":[
    {"pts_time":"0","duration_time":"0.083333"},
    {"pts_time":"0.916667","duration_time":"0.083333"}]}'
  negative='{"streams":[{"start_time":"-0.1","duration":"1"}],"packets":[
    {"pts_time":"-0.1","duration_time":"0.083333"},
    {"pts_time":"0.916667","duration_time":"0.083333"}]}'
  positive='{"streams":[{"start_time":"0.1","duration":"1"}],"packets":[
    {"pts_time":"0.1","duration_time":"0.083333"},
    {"pts_time":"0.916667","duration_time":"0.083333"}]}'
  short='{"streams":[{"start_time":"0"}],"packets":[
    {"pts_time":"0","duration_time":"0.083333"},
    {"pts_time":"0.75","duration_time":"0.083333"}]}'
  assert_smoke_interval_fixture "$valid"
  expect_smoke_interval_failure negative-start "$negative"
  expect_smoke_interval_failure positive-start "$positive"
  expect_smoke_interval_failure short-packets "$short"
}

check_smoke_config_contracts() {
  local root=$1 config dir long_config
  for config in out_main out_4k; do
    dir="$root/config-$config"
    write_smoke_project "$dir" null
    make_smoke_video "$dir/rendered/preview.mp4" '#203040'
    jq --arg id "$config" '.project.render_configs[0].id=$id' \
      "$dir/project/project.preview.veac.json" >"$dir/project.tmp"
    mv "$dir/project.tmp" "$dir/project/project.preview.veac.json"
    jq --arg id "$config" '.output.render_config_id=$id' \
      "$dir/plans/preview/out_preview.json" >"$dir/plans/preview/$config.json"
    rm "$dir/plans/preview/out_preview.json"
    check_example_media_smoke "$dir"
  done
  dir="$root/config-uppercase"
  write_smoke_project "$dir" null
  make_smoke_video "$dir/rendered/preview.mp4" '#203040'
  jq '.project.render_configs[0].id="out_Main"' \
    "$dir/project/project.preview.veac.json" >"$dir/project.tmp"
  mv "$dir/project.tmp" "$dir/project/project.preview.veac.json"
  expect_failure config-uppercase "$dir"
  dir="$root/config-too-long"
  write_smoke_project "$dir" null
  make_smoke_video "$dir/rendered/preview.mp4" '#203040'
  long_config=$(awk 'BEGIN { printf "out_"; for (i=0; i<125; i++) printf "a" }')
  jq --arg id "$long_config" '.project.render_configs[0].id=$id' \
    "$dir/project/project.preview.veac.json" >"$dir/project.tmp"
  mv "$dir/project.tmp" "$dir/project/project.preview.veac.json"
  expect_failure config-too-long "$dir"
}

check_smoke_ancestor_contract() {
  local root=$1 catalog=$2 real="$1/real-parent" link="$1/linked-parent"
  mkdir -p "$real/preview/minimal"
  ln -s "$real" "$link"
  if (VEAC_EXAMPLES=minimal assert_smoke_roster "$link/preview" \
      "$catalog") >/dev/null 2>&1; then
    fail "symlink preview ancestor passed"
  fi
}

run_media_smoke_boundary_contracts() {
  local root=$1 catalog=$2
  check_smoke_ebml_contracts "$root"
  check_smoke_interval_contracts
  check_smoke_config_contracts "$root"
  check_smoke_ancestor_contract "$root" "$catalog"
}

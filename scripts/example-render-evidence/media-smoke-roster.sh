#!/usr/bin/env bash

smoke_expected_ids() {
  local catalog=$1 selector=${VEAC_EXAMPLES:-} id
  selector=${selector//,/ }
  if [[ -z $selector || $selector == all ]]; then
    jq -er '.targets[] | select(.example != null) | .id' "$catalog" | LC_ALL=C sort
    return
  fi
  for id in $selector; do
    [[ $id =~ ^[a-z0-9][a-z0-9-]*$ ]] || fail "invalid example selector: $id"
    jq -e --arg id "$id" 'any(.targets[]; .id == $id and .example != null)' \
      "$catalog" >/dev/null || fail "unknown selected example: $id"
    printf '%s\n' "$id"
  done | LC_ALL=C sort -u
}

smoke_actual_ids() {
  local root=$1 dir id
  local directories=()
  shopt -s nullglob dotglob
  directories=("$root"/*/)
  shopt -u nullglob dotglob
  for dir in "${directories[@]}"; do
    id=$(basename "${dir%/}")
    [[ $id == .fixtures ]] && continue
    smoke_assert_directory "${dir%/}" "example directory $id"
    printf '%s\n' "$id"
  done | LC_ALL=C sort
}

assert_smoke_roster() {
  local root=$1 catalog=$2 expected actual
  smoke_assert_directory "$root" "preview root"
  expected=$(smoke_expected_ids "$catalog") || fail "cannot resolve expected examples"
  actual=$(smoke_actual_ids "$root") || fail "cannot inventory rendered examples"
  [[ -n $expected && $actual == "$expected" ]] ||
    fail "rendered example directories do not match the selected catalog roster"
}

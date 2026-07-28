#!/usr/bin/env bash

prepare_example_preview_cli() {
  local root=$1
  local toolchain=$2
  local target_dir

  if [[ ${VEAC_BIN+x} == x ]]; then
    [[ -n "$VEAC_BIN" ]] || {
      echo "example preview CLI: VEAC_BIN must not be empty" >&2
      return 2
    }
    printf '%s\n' "$VEAC_BIN"
    return
  fi

  command -v cargo >/dev/null 2>&1 || {
    echo "example preview CLI: required command not found: cargo" >&2
    return 1
  }
  cargo +"$toolchain" build --manifest-path "$root/Cargo.toml" \
    --package veac-cli --bin veac >&2
  target_dir=$(cargo +"$toolchain" metadata --manifest-path "$root/Cargo.toml" \
    --format-version 1 --no-deps | jq -r '.target_directory')
  printf '%s/debug/veac\n' "$target_dir"
}

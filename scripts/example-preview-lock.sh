#!/usr/bin/env bash

PREVIEW_LOCK=""

release_example_preview_lock() {
  [[ -n "$PREVIEW_LOCK" && -f "$PREVIEW_LOCK/pid" ]] || return 0
  [[ $(< "$PREVIEW_LOCK/pid") == "$$" ]] || return 0
  rm -f "$PREVIEW_LOCK/pid"
  rmdir "$PREVIEW_LOCK" 2>/dev/null || true
}

acquire_example_preview_lock() {
  local root=$1
  local lock="$root/target/.examples-preview.lock"
  local owner=""
  mkdir -p "$root/target"
  if ! mkdir "$lock" 2>/dev/null; then
    [[ ! -L "$lock" ]] || {
      echo "examples preview: lock path must not be a symlink" >&2
      return 1
    }
    [[ -f "$lock/pid" ]] && owner=$(< "$lock/pid")
    if [[ "$owner" =~ ^[0-9]+$ ]] && kill -0 "$owner" 2>/dev/null; then
      echo "examples preview: another build is running in process $owner" >&2
      return 1
    fi
    rm -f "$lock/pid"
    rmdir "$lock" 2>/dev/null || {
      echo "examples preview: stale lock cannot be removed" >&2
      return 1
    }
    mkdir "$lock"
  fi
  PREVIEW_LOCK=$lock
  printf '%s\n' "$$" > "$PREVIEW_LOCK/pid"
}

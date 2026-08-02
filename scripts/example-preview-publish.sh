#!/usr/bin/env bash

PREVIEW_STAGING=""
PREVIEW_BACKUP=""

preview_publish_error() {
  echo "examples preview: $*" >&2
  return 1
}

preview_scratch_allowed() {
  local root=$1
  local output=$2
  local path=$3
  local relative
  case "$output" in
    "$root/examples-preview"|"$root/examples-preview/"*) ;;
    *) return 1 ;;
  esac
  [[ $(dirname "$path") == "$(dirname "$output")" ]] || return 1
  case "$path" in "$output.staging.$$"|"$output.previous.$$") ;; *) return 1 ;; esac
  relative=${path#"$root"/}
  git -C "$root" check-ignore -q -- "$relative/"
}

remove_preview_scratch() {
  local root=$1
  local output=$2
  local path=$3
  [[ -z "$path" ]] && return 0
  if ! preview_scratch_allowed "$root" "$output" "$path"; then
    preview_publish_error "refusing to remove unsafe scratch path: $path"
    return 1
  fi
  if [[ -L "$path" ]]; then
    preview_publish_error "scratch path must not be a symlink: $path"
    return 1
  fi
  rm -rf "$path" || return 1
}

create_preview_staging() {
  local root=$1
  local output=$2
  PREVIEW_STAGING="$output.staging.$$"
  PREVIEW_BACKUP="$output.previous.$$"
  if [[ -L "$output" || (-e "$output" && ! -d "$output") ]]; then
    preview_publish_error "preview output must be a non-symlink directory"
    return 1
  fi
  if ! preview_scratch_allowed "$root" "$output" "$PREVIEW_STAGING" ||
      ! preview_scratch_allowed "$root" "$output" "$PREVIEW_BACKUP"; then
    preview_publish_error "scratch paths are outside the ignored preview boundary"
    return 1
  fi
  if [[ -e "$PREVIEW_STAGING" || -L "$PREVIEW_STAGING" ||
        -e "$PREVIEW_BACKUP" || -L "$PREVIEW_BACKUP" ]]; then
    preview_publish_error "preview transaction scratch path already exists"
    return 1
  fi
  mkdir "$PREVIEW_STAGING" || return 1
}

publish_preview_staging() {
  local root=$1
  local output=$2
  if ! preview_scratch_allowed "$root" "$output" "$PREVIEW_STAGING" ||
      [[ ! -d "$PREVIEW_STAGING" || -L "$PREVIEW_STAGING" || -L "$output" ]]; then
    preview_publish_error "invalid preview staging transaction"
    return 1
  fi
  if [[ -e "$output" ]] && ! mv "$output" "$PREVIEW_BACKUP"; then
    preview_publish_error "failed to preserve previous preview output"
    return 1
  fi
  if ! mv "$PREVIEW_STAGING" "$output"; then
    if [[ -d "$PREVIEW_BACKUP" && ! -e "$output" ]]; then
      mv "$PREVIEW_BACKUP" "$output" ||
        preview_publish_error "failed to restore previous preview output"
    fi
    preview_publish_error "failed to publish staged preview output"
  fi
  PREVIEW_STAGING=""
  remove_preview_scratch "$root" "$output" "$PREVIEW_BACKUP" || return 1
  PREVIEW_BACKUP=""
}

cleanup_preview_transaction() {
  local root=$1
  local output=$2
  if [[ -n "$PREVIEW_BACKUP" && (-e "$PREVIEW_BACKUP" || -L "$PREVIEW_BACKUP") ]]; then
    if [[ ! -e "$output" ]]; then
      [[ -d "$PREVIEW_BACKUP" && ! -L "$PREVIEW_BACKUP" ]] || return 1
      mv "$PREVIEW_BACKUP" "$output" || return 1
    else
      remove_preview_scratch "$root" "$output" "$PREVIEW_BACKUP" || return 1
    fi
  fi
  PREVIEW_BACKUP=""
  remove_preview_scratch "$root" "$output" "$PREVIEW_STAGING"
  PREVIEW_STAGING=""
}

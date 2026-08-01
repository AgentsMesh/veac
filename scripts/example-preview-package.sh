#!/usr/bin/env bash

preview_package_safe_relative() {
  local path=$1
  [[ -n "$path" && "$path" != /* && "$path" =~ ^[A-Za-z0-9._/-]+$ ]] || return 1
  case "/$path/" in
    *"//"*|*"/./"*|*"/../"*) return 1 ;;
  esac
}

verify_preview_package_reference() {
  local root=$1 playlist=$2 reference=$3 base relative
  if ! preview_package_safe_relative "$reference"; then
    preview_artifact_error "unsafe HLS reference in $playlist: $reference"
    return 1
  fi
  base=${playlist%/*}
  [[ "$base" != "$playlist" ]] || base=
  relative=${base:+"$base/"}$reference
  if ! preview_package_safe_relative "$relative" ||
     [[ ! -f "$root/$relative" || -L "$root/$relative" ]]; then
    preview_artifact_error "missing HLS package member: $relative"
    return 1
  fi
}

verify_preview_hls_playlist() {
  local root=$1 playlist=$2 first line reference
  local uri_pattern='URI="([^"]+)"'
  if ! IFS= read -r first < "$root/$playlist"; then
    preview_artifact_error "empty HLS playlist: $playlist"
    return 1
  fi
  first=${first%$'\r'}
  if [[ "$first" != '#EXTM3U' ]]; then
    preview_artifact_error "invalid HLS playlist header: $playlist"
    return 1
  fi
  while IFS= read -r line || [[ -n "$line" ]]; do
    line=${line%$'\r'}
    reference=
    if [[ "$line" == \#* && "$line" =~ $uri_pattern ]]; then
      reference=${BASH_REMATCH[1]}
    elif [[ -n "$line" && "$line" != \#* ]]; then
      reference=$line
    fi
    [[ -z "$reference" ]] ||
      verify_preview_package_reference "$root" "$playlist" "$reference" || return 1
  done < "$root/$playlist"
}

verify_preview_package_tree() {
  local root=$1 inventory member relative status=0
  inventory=$(mktemp "${TMPDIR:-/tmp}/veac-preview-package.XXXXXX") || {
    preview_artifact_error "cannot allocate package inventory"
    return 1
  }
  if ! find -P "$root" -mindepth 1 -print0 > "$inventory"; then
    rm -f "$inventory"
    preview_artifact_error "cannot enumerate delivery package: $root"
    return 1
  fi
  while IFS= read -r -d '' member; do
    relative=${member#"$root"/}
    if ! preview_package_safe_relative "$relative"; then
      preview_artifact_error "unsafe delivery package member: $relative"
      status=1
    elif [[ -L "$member" || (! -f "$member" && ! -d "$member") ]]; then
      preview_artifact_error "delivery package member is not a regular file or directory: $relative"
      status=1
    elif [[ -f "$member" && ! -s "$member" ]]; then
      preview_artifact_error "empty delivery package member: $relative"
      status=1
    elif [[ -f "$member" && "$relative" == *.m3u8 ]] &&
         ! verify_preview_hls_playlist "$root" "$relative"; then
      status=1
    fi
    [[ $status -eq 0 ]] || break
  done < "$inventory"
  rm -f "$inventory"
  return "$status"
}

verify_preview_package() {
  local rendered=$1 name=$2 kind=$3 package_kind=$4 root entrypoint format
  if [[ "$kind" != adaptive_package || "$package_kind" != hls ]]; then
    preview_artifact_error "unsupported package deliverable kind: $kind/$package_kind"
    return 1
  fi
  if ! preview_package_safe_relative "$name" || [[ "$name" == */* ]]; then
    preview_artifact_error "unsafe package deliverable name: $name"
    return 1
  fi
  root="$rendered/$name"
  if [[ ! -d "$root" || -L "$root" ]]; then
    preview_artifact_error "missing package deliverable directory: $root"
    return 1
  fi
  entrypoint="$root/master.m3u8"
  if [[ ! -s "$entrypoint" || ! -f "$entrypoint" || -L "$entrypoint" ]]; then
    preview_artifact_error "missing HLS package entrypoint: $entrypoint"
    return 1
  fi
  verify_preview_package_tree "$root" || return 1
  if ! format=$(ffprobe -v error -show_entries format=format_name \
    -of default=noprint_wrappers=1:nokey=1 "$entrypoint"); then
    preview_artifact_error "unreadable HLS package entrypoint: $entrypoint"
    return 1
  fi
  if [[ "$format" != hls ]]; then
    preview_artifact_error "package entrypoint is not HLS: $entrypoint"
    return 1
  fi
  ffmpeg -v error -xerror -nostdin -i "$entrypoint" -map 0:v:0 \
    -frames:v 1 -f null - >/dev/null 2>&1 || {
    preview_artifact_error "HLS package cannot decode a video frame: $entrypoint"
    return 1
  }
}

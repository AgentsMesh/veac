#!/usr/bin/env bash

stage_example_source_graph() {
  local source_root=$1 destination_root=$2 path relative destination count=0
  [[ -d $source_root && ! -L $source_root ]] || {
    echo "example source root must be a regular directory: $source_root" >&2
    return 1
  }
  if find "$source_root" -type l -print -quit | grep -q .; then
    echo "example source graph must not contain symlinks: $source_root" >&2
    return 1
  fi
  while IFS= read -r -d '' path; do
    relative=${path#"$source_root"/}
    [[ $relative =~ ^[A-Za-z0-9_./-]+\.veac$ ]] || {
      echo "example module path is not portable: $relative" >&2
      return 1
    }
    destination="$destination_root/$relative"
    mkdir -p "$(dirname "$destination")"
    cp "$path" "$destination"
    count=$((count + 1))
  done < <(find "$source_root" -type f -name '*.veac' -print0)
  [[ $count -gt 0 && -f $destination_root/main.veac ]] || {
    echo "example source graph requires main.veac: $source_root" >&2
    return 1
  }
}

verify_example_source_graph() {
  local source_root=$1 destination_root=$2 path relative source_count=0 staged_count
  while IFS= read -r -d '' path; do
    relative=${path#"$source_root"/}
    cmp -s "$path" "$destination_root/$relative" || {
      echo "example source copy drifted: $relative" >&2
      return 1
    }
    source_count=$((source_count + 1))
  done < <(find "$source_root" -type f -name '*.veac' -print0)
  staged_count=$(find "$destination_root" -type f -name '*.veac' | wc -l | tr -d ' ')
  [[ $source_count == "$staged_count" ]] || {
    echo "example staged source graph contains unexpected modules" >&2
    return 1
  }
}

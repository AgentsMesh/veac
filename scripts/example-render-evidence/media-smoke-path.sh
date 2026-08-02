#!/usr/bin/env bash

smoke_assert_no_symlink_ancestors() {
  local path=$1 label=$2 absolute current component
  local components=()
  [[ -n $path ]] || fail "$label path is empty"
  case $path in
    /*) absolute=$path ;;
    *) absolute="$PWD/$path" ;;
  esac
  IFS=/ read -r -a components <<< "$absolute"
  current=
  for component in "${components[@]}"; do
    [[ -n $component && $component != . ]] || continue
    current="$current/$component"
    [[ ! -L $current ]] || fail "$label has a symlink path component: $current"
  done
}

smoke_assert_directory() {
  local path=$1 label=$2
  smoke_assert_no_symlink_ancestors "$path" "$label"
  [[ -d $path ]] || fail "$label is not a directory: $path"
}

smoke_assert_regular_file() {
  local path=$1 label=$2
  smoke_assert_no_symlink_ancestors "$path" "$label"
  [[ -f $path ]] || fail "$label is not a regular file: $path"
}

smoke_assert_nonempty_file() {
  local path=$1 label=$2
  smoke_assert_regular_file "$path" "$label"
  [[ -s $path ]] || fail "$label is empty: $path"
}

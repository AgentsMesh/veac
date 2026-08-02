#!/usr/bin/env bash

PREVIEW_PROCESS_PID=""

stop_example_preview_process() {
  local child
  local children=""
  [[ "$PREVIEW_PROCESS_PID" =~ ^[0-9]+$ ]] || return 0
  children=$(pgrep -P "$PREVIEW_PROCESS_PID" 2>/dev/null || true)
  for child in $children; do kill -TERM "$child" 2>/dev/null || true; done
  kill -TERM "$PREVIEW_PROCESS_PID" 2>/dev/null || true
  sleep 0.2
  for child in $children; do kill -KILL "$child" 2>/dev/null || true; done
  kill -KILL "$PREVIEW_PROCESS_PID" 2>/dev/null || true
  wait "$PREVIEW_PROCESS_PID" 2>/dev/null || true
  PREVIEW_PROCESS_PID=""
}

run_example_preview_process() {
  local workdir=$1
  local status
  shift
  (cd "$workdir" && exec "$@") &
  PREVIEW_PROCESS_PID=$!
  if wait "$PREVIEW_PROCESS_PID"; then status=0; else status=$?; fi
  PREVIEW_PROCESS_PID=""
  return "$status"
}

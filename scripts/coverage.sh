#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
REPOSITORY_ROOT=$(cd "$SCRIPT_DIR/.." && pwd)
# shellcheck source=coverage-policy.sh
source "$SCRIPT_DIR/coverage-policy.sh"
cd "$REPOSITORY_ROOT"

readonly LLVM_COV_VERSION="0.8.4"

release_coverage_lock() {
  local lock=target/.coverage.lock
  [[ -f "$lock/pid" ]] || return 0
  [[ $(< "$lock/pid") == "$$" ]] || return 0
  rm -f "$lock/pid"
  rmdir "$lock" 2>/dev/null || true
}

acquire_coverage_lock() {
  local lock=target/.coverage.lock
  local stale="${lock}.stale.$$"
  local owner=""
  mkdir -p target
  if ! mkdir "$lock" 2>/dev/null; then
    [[ -f "$lock/pid" ]] && owner=$(< "$lock/pid")
    if [[ "$owner" =~ ^[0-9]+$ ]] && kill -0 "$owner" 2>/dev/null; then
      echo "error: coverage is already running in process $owner" >&2
      exit 2
    fi
    mv "$lock" "$stale" 2>/dev/null || {
      echo "error: another coverage process acquired the lock" >&2
      exit 2
    }
    rm -f "$stale/pid"
    rmdir "$stale"
    mkdir "$lock" 2>/dev/null || {
      echo "error: another coverage process acquired the lock" >&2
      exit 2
    }
  fi
  printf '%s\n' "$$" > "$lock/pid"
  trap release_coverage_lock EXIT
}

require_llvm_cov_version() {
  local actual
  actual=$(cargo llvm-cov --version)
  if [[ "$actual" != "cargo-llvm-cov $LLVM_COV_VERSION" ]]; then
    echo "error: expected cargo-llvm-cov $LLVM_COV_VERSION, found $actual" >&2
    return 2
  fi
}

coverage_packages() {
  cargo metadata --no-deps --format-version 1 | python3 -c '
import json
import sys

metadata = json.load(sys.stdin)
members = set(metadata["workspace_members"])
names = sorted(package["name"] for package in metadata["packages"] if package["id"] in members)
print("\n".join(names))
'
}

report_gate() {
  cargo llvm-cov report "$@" \
    --ignore-filename-regex "$COVERAGE_TEST_SOURCE_REGEX" \
    --fail-under-lines "$COVERAGE_MINIMUM" \
    --fail-under-functions "$COVERAGE_MINIMUM"
}

run_package() {
  local package=$1
  local known=1
  local candidate
  while IFS= read -r candidate; do
    if [[ "$candidate" == "$package" ]]; then
      known=0
      break
    fi
  done < <(coverage_packages)
  if (( known )); then
    echo "error: unknown workspace package: $package" >&2
    return 2
  fi
  export CARGO_TARGET_DIR="${COVERAGE_TARGET_ROOT:-target}/coverage-$package"
  cargo llvm-cov clean --workspace
    cargo llvm-cov -p "$package" --all-features --lib --bins --tests --no-report \
      -- --test-threads=1
  report_gate -p "$package"
}

run_packages() {
  local package
  while IFS= read -r package; do
    run_package "$package"
  done < <(coverage_packages)
}

run_workspace() {
  local report_path=${1:-target/coverage/lcov.info}
  export CARGO_TARGET_DIR="${COVERAGE_TARGET_ROOT:-target}/coverage-workspace"
  rm -f "$report_path"
  cargo llvm-cov clean --workspace
  cargo llvm-cov --workspace --all-features --all-targets --no-report \
    -- --test-threads=1
  mkdir -p "$(dirname "$report_path")"
  cargo llvm-cov report \
    --ignore-filename-regex "$COVERAGE_TEST_SOURCE_REGEX" \
    --lcov --output-path "$report_path"
  report_gate
  local package
  while IFS= read -r package; do
    report_gate -p "$package"
  done < <(coverage_packages)
}

case ${1:-} in
  package)
    [[ $# -eq 2 ]] || { echo "usage: $0 package <name>" >&2; exit 2; }
    require_llvm_cov_version
    acquire_coverage_lock
    run_package "$2"
    ;;
  packages)
    [[ $# -eq 1 ]] || { echo "usage: $0 packages" >&2; exit 2; }
    require_llvm_cov_version
    acquire_coverage_lock
    run_packages
    ;;
  workspace)
    [[ $# -le 2 ]] || { echo "usage: $0 workspace [lcov-path]" >&2; exit 2; }
    require_llvm_cov_version
    acquire_coverage_lock
    run_workspace "${2:-}"
    ;;
  *)
    echo "usage: $0 {package <name>|packages|workspace [lcov-path]}" >&2
    exit 2
    ;;
esac

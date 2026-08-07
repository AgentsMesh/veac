#!/usr/bin/env bash
set -euo pipefail

archive=${1:?usage: smoke-release-archive.sh ARCHIVE TARGET RELEASE}
target=${2:?usage: smoke-release-archive.sh ARCHIVE TARGET RELEASE}
release=${3:?usage: smoke-release-archive.sh ARCHIVE TARGET RELEASE}
root="veac-${release}-${target}"
expected_archive="$root.tar.gz"
[[ -f $archive && $(basename "$archive") == "$expected_archive" ]] || {
  echo "release smoke: unexpected archive: $archive" >&2
  exit 1
}

temp=$(mktemp -d)
trap 'rm -rf "$temp"' EXIT
listing="$temp/archive.list"
types="$temp/archive.types"
tar tzf "$archive" >"$listing"
tar tvzf "$archive" >"$types"
awk '$1 !~ /^[-d]/ { bad = 1 } END { exit (NR == 0 || bad) }' "$types" || {
  echo "release smoke: archive may contain only regular files and directories" >&2
  exit 1
}
found_binary=0
found_readme=0
found_license=0
while IFS= read -r entry; do
  case "/$entry/" in
    *'/../'*|*'/./'*) echo "release smoke: unsafe archive entry: $entry" >&2; exit 1 ;;
  esac
  case "$entry" in
    "$root"|"$root/"|"$root/"*) ;;
    *) echo "release smoke: entry escapes package root: $entry" >&2; exit 1 ;;
  esac
  [[ $entry == "$root/veac" ]] && found_binary=$((found_binary + 1))
  [[ $entry == "$root/README.md" ]] && found_readme=$((found_readme + 1))
  [[ $entry == "$root/LICENSE" ]] && found_license=$((found_license + 1))
done <"$listing"
[[ $found_binary == 1 ]] || { echo "release smoke: expected one packaged veac" >&2; exit 1; }
[[ $found_readme == 1 && $found_license == 1 ]] || {
  echo "release smoke: packaged documentation is incomplete" >&2
  exit 1
}

tar xzf "$archive" -C "$temp"
binary="$temp/$root/veac"
[[ -f $binary && -x $binary && ! -L $binary ]] || {
  echo "release smoke: packaged veac is not a regular executable" >&2
  exit 1
}

"$binary" --help >"$temp/help.txt"
[[ -s $temp/help.txt ]] || { echo "release smoke: --help was empty" >&2; exit 1; }
"$binary" language-spec >"$temp/language-spec.json"
"$binary" language-spec --schema >"$temp/language-spec.schema.json"

python3 - "$temp/language-spec.json" "$temp/language-spec.schema.json" "${release#v}" <<'PY'
import json
import pathlib
import sys


def canonical(path):
    raw = pathlib.Path(path).read_bytes()
    try:
        value = json.loads(raw)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise SystemExit(f"release smoke: invalid JSON from language-spec: {error}")
    encoded = json.dumps(
        value, ensure_ascii=False, separators=(",", ":"), sort_keys=True
    ).encode() + b"\n"
    if raw != encoded:
        raise SystemExit("release smoke: language-spec output is not canonical JSON")
    return value


spec = canonical(sys.argv[1])
schema = canonical(sys.argv[2])
version = sys.argv[3]
identity = ("https://veac.dev/schemas/language-spec", 7, "veac", version)
if tuple(spec.get(key) for key in
         ("schema", "schema_version", "language", "language_version")) != identity:
    raise SystemExit("release smoke: language-spec build identity mismatch")
properties = schema.get("properties", {})
if tuple(properties.get(key, {}).get("const") for key in
         ("schema", "schema_version", "language", "language_version")) != identity:
    raise SystemExit("release smoke: language-spec schema identity mismatch")
PY

echo "release archive smoke passed: $expected_archive"

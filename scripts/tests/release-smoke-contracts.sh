#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)
TEMP=$(mktemp -d)
trap 'rm -rf "$TEMP"' EXIT
RELEASE=v0.1.0
TARGET=x86_64-unknown-linux-gnu
PACKAGE="veac-${RELEASE}-${TARGET}"
mkdir -p "$TEMP/stage/$PACKAGE"
printf 'readme\n' >"$TEMP/stage/$PACKAGE/README.md"
printf 'license\n' >"$TEMP/stage/$PACKAGE/LICENSE"

cat >"$TEMP/stage/$PACKAGE/veac" <<'SH'
#!/bin/sh
case "${1:-}" in
  --help) echo 'VEAC command help' ;;
  language-spec)
    version=0.1.0
    [ "${FAKE_BAD_IDENTITY:-0}" = 0 ] || version=9.9.9
    if [ "${2:-}" = --schema ]; then
      printf '%s\n' "{\"properties\":{\"language\":{\"const\":\"veac\"},\"language_version\":{\"const\":\"$version\"},\"schema\":{\"const\":\"https://veac.dev/schemas/language-spec\"},\"schema_version\":{\"const\":7}}}"
    elif [ "${FAKE_NONCANONICAL:-0}" = 1 ]; then
      printf '%s\n' '{ "language": "veac" }'
    else
      printf '%s\n' "{\"language\":\"veac\",\"language_version\":\"$version\",\"schema\":\"https://veac.dev/schemas/language-spec\",\"schema_version\":7}"
    fi
    ;;
  *) exit 2 ;;
esac
SH
chmod +x "$TEMP/stage/$PACKAGE/veac"
ARCHIVE="$TEMP/$PACKAGE.tar.gz"
tar czf "$ARCHIVE" -C "$TEMP/stage" "$PACKAGE"
SMOKE="$ROOT/.github/scripts/smoke-release-archive.sh"

bash "$SMOKE" "$ARCHIVE" "$TARGET" "$RELEASE" >"$TEMP/valid.log"
rg -q 'release archive smoke passed' "$TEMP/valid.log"

mkdir -p "$TEMP/duplicate"
python3 - "$TEMP/stage" "$TEMP/duplicate/$PACKAGE.tar.gz" "$PACKAGE" <<'PY'
import pathlib
import sys
import tarfile

stage = pathlib.Path(sys.argv[1])
archive = pathlib.Path(sys.argv[2])
package = sys.argv[3]
with tarfile.open(archive, "w:gz") as output:
    output.add(stage / package, arcname=package)
    output.add(stage / package / "veac", arcname=f"{package}/veac")
PY
if bash "$SMOKE" "$TEMP/duplicate/$PACKAGE.tar.gz" "$TARGET" "$RELEASE" \
    >"$TEMP/duplicate.log" 2>&1; then
  echo "release smoke accepted a duplicate packaged binary" >&2
  exit 1
fi
rg -q 'expected one packaged veac' "$TEMP/duplicate.log"

mkdir -p "$TEMP/incomplete-stage/$PACKAGE" "$TEMP/incomplete"
cp "$TEMP/stage/$PACKAGE/veac" "$TEMP/stage/$PACKAGE/README.md" \
  "$TEMP/incomplete-stage/$PACKAGE/"
tar czf "$TEMP/incomplete/$PACKAGE.tar.gz" -C "$TEMP/incomplete-stage" "$PACKAGE"
if bash "$SMOKE" "$TEMP/incomplete/$PACKAGE.tar.gz" "$TARGET" "$RELEASE" \
    >"$TEMP/incomplete.log" 2>&1; then
  echo "release smoke accepted incomplete packaged documentation" >&2
  exit 1
fi
rg -q 'packaged documentation is incomplete' "$TEMP/incomplete.log"

if FAKE_BAD_IDENTITY=1 bash "$SMOKE" "$ARCHIVE" "$TARGET" "$RELEASE" \
    >"$TEMP/bad-identity.log" 2>&1; then
  echo "release smoke accepted a stale language identity" >&2
  exit 1
fi
rg -q 'build identity mismatch' "$TEMP/bad-identity.log"

if FAKE_NONCANONICAL=1 bash "$SMOKE" "$ARCHIVE" "$TARGET" "$RELEASE" \
    >"$TEMP/noncanonical.log" 2>&1; then
  echo "release smoke accepted non-canonical language-spec JSON" >&2
  exit 1
fi
rg -q 'not canonical JSON' "$TEMP/noncanonical.log"

echo "release archive smoke contracts passed"

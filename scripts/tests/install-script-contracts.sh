#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)
rg -Fq 'curl -fsSL https://raw.githubusercontent.com/AgentsMesh/veac/main/install.sh | env VEAC_VERSION=v0.1.0 sh' \
  "$ROOT/README.md" || { echo "versioned installer example does not pass VEAC_VERSION to sh" >&2; exit 1; }
TEMP=$(mktemp -d)
trap 'rm -rf "$TEMP"' EXIT
MOCK="$TEMP/mock-bin"
mkdir -p "$MOCK"
cat >"$MOCK/uname" <<'SH'
#!/bin/sh
case "$1" in
  -s) echo Linux ;;
  -m) echo x86_64 ;;
  *) exit 2 ;;
esac
SH
cat >"$MOCK/curl" <<'SH'
#!/bin/sh
output=
url=
while [ "$#" -gt 0 ]; do
  case "$1" in
    -o) output=$2; shift 2 ;;
    -*) shift ;;
    *) url=$1; shift ;;
  esac
done
case "$url" in
  *api.github.com*/releases/latest) printf '{"tag_name":"%s"}\n' "$LATEST_VERSION" ;;
  *checksums-sha256.txt)
    [ "${CHECKSUM_UNAVAILABLE:-0}" = 0 ] || exit 22
    cp "$CHECKSUM_FIXTURE" "$output"
    ;;
  *.tar.gz) cp "$ARCHIVE_FIXTURE" "$output" ;;
  *) exit 22 ;;
esac
SH
cat >"$MOCK/sha256sum" <<'SH'
#!/bin/sh
[ "$#" -eq 1 ] || exit 2
if [ "${HASH_MODE:-valid}" = malformed ]; then
  printf '%s  %s\n' "$ARCHIVE_DIGEST" wrong-file
  exit 0
fi
printf '%s  %s\n' "$ARCHIVE_DIGEST" "$1"
SH
chmod +x "$MOCK/uname" "$MOCK/curl" "$MOCK/sha256sum"

FALLBACK="$TEMP/fallback-bin"
mkdir -p "$FALLBACK"
for command in awk chmod cp gzip mkdir mktemp rm tar; do
  ln -s "$(command -v "$command")" "$FALLBACK/$command"
done
ln -s "$MOCK/uname" "$FALLBACK/uname"
ln -s "$MOCK/curl" "$FALLBACK/curl"
cat >"$FALLBACK/shasum" <<'SH'
#!/bin/sh
[ "$1" = -a ] && [ "$2" = 256 ] && [ "$#" -eq 3 ] || exit 2
printf '%s  %s\n' "$ARCHIVE_DIGEST" "$3"
SH
chmod +x "$FALLBACK/shasum"

VERSION=v9.8.7
TARGET=x86_64-unknown-linux-gnu
NAME="veac-${VERSION}-${TARGET}"
DIGEST=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
PAYLOAD="$TEMP/payload"
mkdir -p "$PAYLOAD/$NAME"
printf '#!/bin/sh\necho veac\n' >"$PAYLOAD/$NAME/veac"
chmod +x "$PAYLOAD/$NAME/veac"
ARCHIVE="$TEMP/$NAME.tar.gz"
tar czf "$ARCHIVE" -C "$PAYLOAD" "$NAME"
MANIFEST="$TEMP/checksums-sha256.txt"

run_installer() {
  local label=$1
  local archive_fixture=${TEST_ARCHIVE:-$ARCHIVE}
  local installer_path=${INSTALLER_PATH:-$MOCK:$PATH}
  CHECKSUM_FIXTURE="$MANIFEST" ARCHIVE_FIXTURE="$archive_fixture" ARCHIVE_DIGEST="$DIGEST" \
    LATEST_VERSION="$VERSION" VEAC_VERSION="${TEST_VERSION-$VERSION}" \
    HASH_MODE="${HASH_MODE:-valid}" \
    CHECKSUM_UNAVAILABLE="${CHECKSUM_UNAVAILABLE:-0}" PATH="$installer_path" \
    VEAC_INSTALL_DIR="$TEMP/install-$label" \
    /bin/sh "$ROOT/install.sh" >"$TEMP/$label.log" 2>&1
}

expect_failure() {
  local label=$1 content=$2
  printf '%s\n' "$content" >"$MANIFEST"
  if run_installer "$label"; then
    echo "installer contract unexpectedly passed: $label" >&2
    exit 1
  fi
  rg -q '\[veac\] ERROR:' "$TEMP/$label.log"
  [[ ! -e "$TEMP/install-$label/veac" ]]
}

expect_failure missing "$DIGEST  another-archive.tar.gz"
expect_failure duplicate "$DIGEST  $NAME.tar.gz
$DIGEST  $NAME.tar.gz"
expect_failure wrong "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb  $NAME.tar.gz"
expect_failure path-injection "$DIGEST  $NAME.tar.gz
$DIGEST  ../$NAME.tar.gz"
expect_failure malformed "$DIGEST $NAME.tar.gz"

printf '%s\n' "$DIGEST  $NAME.tar.gz" >"$MANIFEST"
CHECKSUM_UNAVAILABLE=1
if run_installer unavailable; then
  echo "installer accepted an unavailable checksum manifest" >&2
  exit 1
fi
unset CHECKSUM_UNAVAILABLE

printf '%s\n' "$DIGEST  $NAME.tar.gz" >"$MANIFEST"
if HASH_MODE=malformed run_installer malformed-tool; then
  echo "installer accepted malformed hash-tool output" >&2
  exit 1
fi

TRAVERSAL="$TEMP/traversal.tar.gz"
python3 - "$TRAVERSAL" "$NAME" <<'PY'
import io
import sys
import tarfile

with tarfile.open(sys.argv[1], "w:gz") as archive:
    for name in ("../outside", f"{sys.argv[2]}/veac"):
        member = tarfile.TarInfo(name)
        member.mode = 0o755
        member.size = 1
        archive.addfile(member, io.BytesIO(b"x"))
PY
if TEST_ARCHIVE="$TRAVERSAL" run_installer unsafe-archive; then
  echo "installer accepted a path-traversing archive" >&2
  exit 1
fi

SYMLINK="$TEMP/symlink.tar.gz"
mkdir -p "$TEMP/symlink/$NAME"
ln -s /bin/sh "$TEMP/symlink/$NAME/veac"
tar czf "$SYMLINK" -C "$TEMP/symlink" "$NAME"
if TEST_ARCHIVE="$SYMLINK" run_installer symlink-binary; then
  echo "installer accepted a symlinked packaged binary" >&2
  exit 1
fi

NONEXEC="$TEMP/nonexec.tar.gz"
mkdir -p "$TEMP/nonexec/$NAME"
printf '#!/bin/sh\n' >"$TEMP/nonexec/$NAME/veac"
chmod 644 "$TEMP/nonexec/$NAME/veac"
tar czf "$NONEXEC" -C "$TEMP/nonexec" "$NAME"
if TEST_ARCHIVE="$NONEXEC" run_installer non-executable; then
  echo "installer accepted a non-executable packaged binary" >&2
  exit 1
fi

INSTALLER_PATH="$FALLBACK" run_installer shasum-fallback
[[ -x "$TEMP/install-shasum-fallback/veac" ]]

NO_HASH="$TEMP/no-hash-bin"
mkdir -p "$NO_HASH"
for command in awk chmod cp gzip mkdir mktemp rm tar; do
  ln -s "$(command -v "$command")" "$NO_HASH/$command"
done
ln -s "$MOCK/uname" "$NO_HASH/uname"
ln -s "$MOCK/curl" "$NO_HASH/curl"
if INSTALLER_PATH="$NO_HASH" run_installer missing-hash-tool; then
  echo "installer skipped checksum verification without a hash tool" >&2
  exit 1
fi

printf '%s\n' "$DIGEST  $NAME.tar.gz" >"$MANIFEST"
victim="$TEMP/symlink-victim"
printf 'do not overwrite\n' >"$victim"
mkdir -p "$TEMP/install-destination-symlink"
ln -s "$victim" "$TEMP/install-destination-symlink/veac"
if run_installer destination-symlink; then
  echo "installer followed an existing destination symlink" >&2
  exit 1
fi
[[ $(<"$victim") == 'do not overwrite' ]]

mkdir -p "$TEMP/install-directory-target/veac"
if run_installer directory-target; then
  echo "installer accepted a non-regular destination" >&2
  exit 1
fi

printf '%s\n' "$DIGEST  another.tar.gz" "$DIGEST  $NAME.tar.gz" >"$MANIFEST"
run_installer valid
[[ -x "$TEMP/install-valid/veac" ]]
TEST_VERSION='' run_installer latest
[[ -x "$TEMP/install-latest/veac" ]]
rg -q 'fetching latest version' "$TEMP/latest.log"

echo "installer checksum contracts passed"

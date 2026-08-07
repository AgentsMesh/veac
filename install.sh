#!/bin/sh
# VEAC installer script
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/AgentsMesh/veac/main/install.sh | sh
#
# Environment variables:
#   VEAC_VERSION     - Version to install (default: latest)
#   VEAC_INSTALL_DIR - Installation directory (default: ~/.local/bin)
set -eu
REPO="AgentsMesh/veac"
BIN_NAME="veac"
DEFAULT_INSTALL_DIR="$HOME/.local/bin"
info() {
    printf '[veac] %s\n' "$@"
}
error() {
    printf '[veac] ERROR: %s\n' "$@" >&2
    exit 1
}
need_cmd() {
    if ! command -v "$1" > /dev/null 2>&1; then
        error "need '$1' (command not found)"
    fi
}
checksum_from_manifest() {
    awk -v wanted="$2" '
        function invalid() { exit 2 }
        {
            digest = substr($0, 1, 64)
            marker = substr($0, 65, 2)
            name = substr($0, 67)
            if (length(digest) != 64 || digest !~ /^[0-9a-f]+$/ ||
                (marker != "  " && marker != " *") ||
                name !~ /^[A-Za-z0-9._-]+$/ || name == "." || name == "..") invalid()
            if (name == wanted) { matches++; expected = digest }
        }
        END { if (matches != 1) exit 3; print expected }
    ' "$1"
}
verify_checksum() {
    manifest=$1
    archive=$2
    name=$3
    [ -s "$manifest" ] || error "checksum manifest is missing or empty"
    expected=$(checksum_from_manifest "$manifest" "$name") ||
        error "checksum manifest must contain exactly one valid entry for '$name'"
    if command -v sha256sum > /dev/null 2>&1; then
        output=$(sha256sum "$archive") || error "could not hash downloaded archive"
    elif command -v shasum > /dev/null 2>&1; then
        output=$(shasum -a 256 "$archive") || error "could not hash downloaded archive"
    else
        error "need 'sha256sum' or 'shasum' to verify the download"
    fi
    actual=${output%% *}
    case "$actual" in
        ''|*[!0-9a-f]*) error "checksum tool returned an invalid digest" ;;
    esac
    [ "${#actual}" -eq 64 ] || error "checksum tool returned an invalid digest"
    [ "$output" = "$actual  $archive" ] || [ "$output" = "$actual *$archive" ] ||
        error "checksum tool returned malformed output"
    [ "$actual" = "$expected" ] ||
        error "checksum verification failed! expected=${expected} actual=${actual}"
}
verify_archive() {
    archive=$1
    root=$2
    names="${TMP_DIR}/archive.names"
    types="${TMP_DIR}/archive.types"
    tar tzf "$archive" > "$names" || error "could not list downloaded archive"
    tar tvzf "$archive" > "$types" || error "could not inspect downloaded archive"
    awk '$1 !~ /^[-d]/ { bad = 1 } END { exit (NR == 0 || bad) }' "$types" ||
        error "archive may contain only regular files and directories"
    binary_count=0
    while IFS= read -r entry; do
        case "/$entry/" in
            *'/../'*|*'/./'*) error "unsafe archive entry: '$entry'" ;;
        esac
        case "$entry" in
            "$root"|"$root/"|"$root/"*) ;;
            *) error "archive entry escapes package root: '$entry'" ;;
        esac
        [ "$entry" = "$root/$BIN_NAME" ] && binary_count=$((binary_count + 1))
    done < "$names"
    [ "$binary_count" -eq 1 ] || error "archive must contain exactly one packaged veac binary"
}
detect_os() {
    case "$(uname -s)" in
        Linux*)  echo "unknown-linux-gnu" ;;
        Darwin*) echo "apple-darwin" ;;
        *)       error "unsupported OS: $(uname -s). Only Linux and macOS are supported." ;;
    esac
}
detect_arch() {
    case "$(uname -m)" in
        x86_64|amd64)   echo "x86_64" ;;
        aarch64|arm64)   echo "aarch64" ;;
        *)               error "unsupported architecture: $(uname -m). Only x86_64 and aarch64/arm64 are supported." ;;
    esac
}
resolve_version() {
    if [ -n "${VEAC_VERSION:-}" ]; then
        echo "$VEAC_VERSION"
        return
    fi
    need_cmd curl
    # stdout is reserved for the version consumed by main's command substitution.
    info "fetching latest version..." >&2
    version=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
        | grep '"tag_name"' \
        | head -1 \
        | sed 's/.*"tag_name": *"\([^"]*\)".*/\1/')
    if [ -z "$version" ]; then
        error "failed to determine latest version. Set VEAC_VERSION manually."
    fi
    echo "$version"
}
TMP_DIR=""
cleanup() { [ -n "$TMP_DIR" ] && rm -rf "$TMP_DIR"; }
main() {
    need_cmd uname
    need_cmd curl
    need_cmd tar
    need_cmd mktemp
    os="$(detect_os)"
    arch="$(detect_arch)"
    target="${arch}-${os}"
    version="$(resolve_version)"
    case "$version" in
        ''|*[!A-Za-z0-9._-]*) error "invalid version: '$version'" ;;
    esac
    install_dir="${VEAC_INSTALL_DIR:-$DEFAULT_INSTALL_DIR}"
    archive_name="${BIN_NAME}-${version}-${target}"
    download_url="https://github.com/${REPO}/releases/download/${version}/${archive_name}.tar.gz"
    checksum_url="https://github.com/${REPO}/releases/download/${version}/checksums-sha256.txt"
    info "platform:  ${target}"
    info "version:   ${version}"
    info "install:   ${install_dir}"
    info ""
    TMP_DIR="$(mktemp -d)"
    trap cleanup EXIT
    info "downloading ${archive_name}.tar.gz ..."
    curl -fsSL "$download_url" -o "${TMP_DIR}/${archive_name}.tar.gz" \
        || error "download failed. Check that version '${version}' exists for target '${target}'."
    info "verifying checksum..."
    curl -fsSL "$checksum_url" -o "${TMP_DIR}/checksums-sha256.txt" 2>/dev/null ||
        error "checksum manifest download failed"
    verify_checksum "${TMP_DIR}/checksums-sha256.txt" \
        "${TMP_DIR}/${archive_name}.tar.gz" "${archive_name}.tar.gz"
    info "checksum OK"
    info "extracting..."
    archive="${TMP_DIR}/${archive_name}.tar.gz"
    verify_archive "$archive" "$archive_name"
    tar xzf "$archive" -C "$TMP_DIR" || error "archive extraction failed"
    binary="${TMP_DIR}/${archive_name}/${BIN_NAME}"
    [ -f "$binary" ] && [ ! -L "$binary" ] && [ -x "$binary" ] ||
        error "packaged veac must be a regular non-symlink executable"
    if [ ! -d "$install_dir" ]; then
        mkdir -p "$install_dir" 2>/dev/null || {
            info "creating ${install_dir} requires elevated permissions"
            sudo mkdir -p "$install_dir"
        }
    fi
    destination="${install_dir}/${BIN_NAME}"
    if [ -e "$destination" ] || [ -L "$destination" ]; then
        [ -f "$destination" ] && [ ! -L "$destination" ] ||
            error "install destination must be a regular non-symlink file"
    fi
    if [ -w "$install_dir" ]; then
        cp "$binary" "$destination"
        chmod +x "$destination"
    else
        info "installing to ${install_dir} requires elevated permissions"
        sudo cp "$binary" "$destination"
        sudo chmod +x "$destination"
    fi
    info ""
    info "veac installed successfully to ${install_dir}/${BIN_NAME}"
    if command -v "$BIN_NAME" > /dev/null 2>&1; then
        info "version: $("$BIN_NAME" --version 2>/dev/null || echo 'installed')"
    else
        info ""
        info "NOTE: '${install_dir}' is not in your PATH."
        info "Add it with:"
        info "  export PATH=\"${install_dir}:\$PATH\""
    fi
}
main

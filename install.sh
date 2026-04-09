#!/bin/sh
# VEAC installer script
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/AgentsMesh/veac/main/install.sh | sh
#
# Environment variables:
#   VEAC_VERSION     - Version to install (default: latest)
#   VEAC_INSTALL_DIR - Installation directory (default: /usr/local/bin)

set -eu

REPO="AgentsMesh/veac"
BIN_NAME="veac"
DEFAULT_INSTALL_DIR="$HOME/.local/bin"

# --- Helper functions ---

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

# --- Detect platform ---

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

# --- Resolve version ---

resolve_version() {
    if [ -n "${VEAC_VERSION:-}" ]; then
        echo "$VEAC_VERSION"
        return
    fi

    need_cmd curl

    info "fetching latest version..."
    version=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
        | grep '"tag_name"' \
        | head -1 \
        | sed 's/.*"tag_name": *"\([^"]*\)".*/\1/')

    if [ -z "$version" ]; then
        error "failed to determine latest version. Set VEAC_VERSION manually."
    fi

    echo "$version"
}

# --- Main ---

main() {
    need_cmd uname
    need_cmd curl
    need_cmd tar
    need_cmd mktemp

    local os arch target version install_dir
    os="$(detect_os)"
    arch="$(detect_arch)"
    target="${arch}-${os}"
    version="$(resolve_version)"
    install_dir="${VEAC_INSTALL_DIR:-$DEFAULT_INSTALL_DIR}"

    local archive_name="${BIN_NAME}-${version}-${target}"
    local download_url="https://github.com/${REPO}/releases/download/${version}/${archive_name}.tar.gz"
    local checksum_url="https://github.com/${REPO}/releases/download/${version}/checksums-sha256.txt"

    info "platform:  ${target}"
    info "version:   ${version}"
    info "install:   ${install_dir}"
    info ""

    # Create temp directory
    local tmp_dir
    tmp_dir="$(mktemp -d)"
    trap 'rm -rf "$tmp_dir"' EXIT

    # Download archive
    info "downloading ${archive_name}.tar.gz ..."
    curl -fsSL "$download_url" -o "${tmp_dir}/${archive_name}.tar.gz" \
        || error "download failed. Check that version '${version}' exists for target '${target}'."

    # Verify checksum
    info "verifying checksum..."
    curl -fsSL "$checksum_url" -o "${tmp_dir}/checksums-sha256.txt" 2>/dev/null
    if [ -f "${tmp_dir}/checksums-sha256.txt" ]; then
        cd "$tmp_dir"
        if command -v sha256sum > /dev/null 2>&1; then
            grep "${archive_name}.tar.gz" checksums-sha256.txt | sha256sum -c --quiet - \
                || error "checksum verification failed!"
        elif command -v shasum > /dev/null 2>&1; then
            expected=$(grep "${archive_name}.tar.gz" checksums-sha256.txt | awk '{print $1}')
            actual=$(shasum -a 256 "${archive_name}.tar.gz" | awk '{print $1}')
            if [ "$expected" != "$actual" ]; then
                error "checksum verification failed! expected=${expected} actual=${actual}"
            fi
        else
            info "warning: no sha256sum or shasum found, skipping checksum verification"
        fi
        cd - > /dev/null
        info "checksum OK"
    else
        info "warning: checksums file not available, skipping verification"
    fi

    # Extract
    info "extracting..."
    tar xzf "${tmp_dir}/${archive_name}.tar.gz" -C "$tmp_dir"

    # Install
    if [ ! -d "$install_dir" ]; then
        mkdir -p "$install_dir" 2>/dev/null || {
            info "creating ${install_dir} requires elevated permissions"
            sudo mkdir -p "$install_dir"
        }
    fi

    if [ -w "$install_dir" ]; then
        cp "${tmp_dir}/${archive_name}/${BIN_NAME}" "${install_dir}/${BIN_NAME}"
        chmod +x "${install_dir}/${BIN_NAME}"
    else
        info "installing to ${install_dir} requires elevated permissions"
        sudo cp "${tmp_dir}/${archive_name}/${BIN_NAME}" "${install_dir}/${BIN_NAME}"
        sudo chmod +x "${install_dir}/${BIN_NAME}"
    fi

    info ""
    info "veac installed successfully to ${install_dir}/${BIN_NAME}"

    # Verify
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

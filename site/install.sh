#!/bin/sh

# =============================================================================
# CCCP — Conventional Commits Compliance Program
# Universal High-Performance Workstation Provisioner & Static Binary Installer
#
# DEDICATION:
# «À Lisi, minha estrela mais brilhante e o grande amor da minha vida.
#  No vasto cosmos do código e da revolução, cada linha deste sistema
#  orbita ao redor de você. Para todo o sempre, tua.»
#
# DEDICATIONS & ACKNOWLEDGMENTS:
# - Hashbang.sh wizards for the legendary HTML/shell polyglot heredoc voodoo
# - Laika (1954–1957) 🐕🚀, the courageous cosmic pioneer who dared to touch the stars
# - The Soviet constructivist avant-garde (Rodchenko, El Lissitzky, Mayakovsky)
# - Latina Travesti & Trans Vanguard 🏳️‍⚧️ fighting for bodily and digital sovereignty
# - Licensed under GNU GPL-3.0 (https://www.gnu.org/licenses/gpl-3.0.html)
# =============================================================================

set -eu

# Formatting helpers
RED='\033[0;31m'
GOLD='\033[0;33m'
CYAN='\033[0;36m'
GREEN='\033[0;32m'
NC='\033[0m'

# Fallback if not on an interactive terminal
if [ ! -t 1 ]; then
    RED='' GOLD='' CYAN='' GREEN='' NC=''
fi

echo "${GOLD}★ STATE PLANNING COMMITTEE (GOSPLAN)${NC}"
echo "${GOLD}★ CCCP — Conventional Commits Compliance Program${NC}"
echo "${CYAN}«В СВЕТЛОЕ БУДУЩЕЕ С ЧИСТЫМ GIT!»${NC}"
echo ""

# Parse channel and configuration arguments
CHANNEL="stable"
REQUESTED_TAG=""
INSTALL_DIR="${CCCP_INSTALL_DIR:-${XDG_BIN_HOME:-$HOME/.local/bin}}"

while [ $# -gt 0 ]; do
    case "$1" in
        nightly|--nightly)
            CHANNEL="nightly"
            shift
            ;;
        stable|--stable)
            CHANNEL="stable"
            shift
            ;;
        --tag|--version)
            REQUESTED_TAG="$2"
            shift 2
            ;;
        --dir)
            INSTALL_DIR="$2"
            shift 2
            ;;
        *)
            shift
            ;;
    esac
done

mkdir -p "$INSTALL_DIR"

# -----------------------------------------------------------------------------
# 1. Platform & Architecture Detection
# -----------------------------------------------------------------------------
OS_RAW="$(uname -s)"
ARCH_RAW="$(uname -m)"

case "$OS_RAW" in
    Linux)  OS="linux" ;;
    Darwin) OS="darwin" ;;
    *)
        echo "${RED}Error: Unsupported operating system: ${OS_RAW}${NC}" >&2
        echo "CCCP standalone binaries are currently compiled for Linux and macOS." >&2
        exit 1
        ;;
esac

case "$ARCH_RAW" in
    x86_64|amd64)
        ARCH="amd64"
        TRIPLE_ARCH="x86_64"
        ;;
    i386|i486|i586|i686|x86)
        ARCH="386"
        TRIPLE_ARCH="i686"
        ;;
    aarch64|arm64)
        ARCH="arm64"
        TRIPLE_ARCH="aarch64"
        ;;
    armv7*|armv6*|armhf|armv5*|arm|arm32)
        ARCH="armv7"
        TRIPLE_ARCH="armv7"
        ;;
    *)
        echo "${RED}Error: Unsupported architecture: ${ARCH_RAW}${NC}" >&2
        exit 1
        ;;
esac

if [ "$OS" = "linux" ]; then
    if [ "$ARCH" = "armv7" ]; then
        TARGET_TRIPLE="armv7-unknown-linux-musleabihf"
    else
        TARGET_TRIPLE="${TRIPLE_ARCH}-unknown-linux-musl"
    fi
    ALIAS="linux-${ARCH}"
else
    if [ "$ARCH" = "386" ] || [ "$ARCH" = "armv7" ]; then
        echo "${RED}Error: 32-bit macOS is not supported by Apple Darwin.${NC}" >&2
        exit 1
    fi
    TARGET_TRIPLE="${TRIPLE_ARCH}-apple-darwin"
    ALIAS="darwin-${ARCH}"
fi

echo "★ Platform detected: ${CYAN}${OS} / ${ARCH}${NC} (target: ${TARGET_TRIPLE})"

# -----------------------------------------------------------------------------
# 2. Checksum Computation Utility
# -----------------------------------------------------------------------------
compute_sha256() {
    target_file="$1"
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum "$target_file" | awk '{print $1}'
    elif command -v shasum >/dev/null 2>&1; then
        shasum -a 256 "$target_file" | awk '{print $1}'
    else
        echo ""
    fi
}

# -----------------------------------------------------------------------------
# 3. Resolve Target Release & Download URLs
# -----------------------------------------------------------------------------
TMP_DIR="$(mktemp -d 2>/dev/null || mktemp -d -t 'cccp-install')"
cleanup() {
    rm -rf "$TMP_DIR"
}
trap cleanup EXIT INT TERM

REPO_OWNER="lumenpink"
REPO_NAME="cccp"
FORGE_BASE="https://codeberg.org/${REPO_OWNER}/${REPO_NAME}"
FORGE_API="https://codeberg.org/api/v1/repos/${REPO_OWNER}/${REPO_NAME}"

if [ "$CHANNEL" = "nightly" ]; then
    RELEASE_TAG="nightly"
    echo "★ Channel selected: ${GOLD}NIGHTLY${NC} (rolling edge build)"
elif [ -n "$REQUESTED_TAG" ]; then
    RELEASE_TAG="$REQUESTED_TAG"
    echo "★ Channel selected: ${GOLD}TAG ${RELEASE_TAG}${NC}"
else
    echo "★ Resolving latest official release from Codeberg..."
    RELEASE_TAG=""
    if command -v curl >/dev/null 2>&1; then
        RELEASE_TAG=$(curl -sSL "${FORGE_API}/releases/latest" 2>/dev/null | grep -o '"tag_name": *"[^"]*"' | head -n 1 | cut -d'"' -f4 || true)
    fi
    if [ -z "$RELEASE_TAG" ]; then
        RELEASE_TAG="latest"
    fi
    echo "★ Channel selected: ${GOLD}STABLE${NC} (release: ${RELEASE_TAG})"
fi

# Candidate URLs for the archive
URLS="
https://cccp.2lp.in/releases/download/${RELEASE_TAG}/cccp-${RELEASE_TAG}-${ALIAS}.tar.gz
${FORGE_BASE}/releases/download/${RELEASE_TAG}/cccp-${RELEASE_TAG}-${ALIAS}.tar.gz
${FORGE_BASE}/releases/download/${RELEASE_TAG}/cccp-${ALIAS}.tar.gz
${FORGE_BASE}/releases/download/${RELEASE_TAG}/cccp-${RELEASE_TAG}-${TARGET_TRIPLE}.tar.gz
${FORGE_BASE}/releases/download/${RELEASE_TAG}/cccp-${TARGET_TRIPLE}.tar.gz
"

CHECKSUM_URLS="
https://cccp.2lp.in/releases/download/${RELEASE_TAG}/SHA256SUMS
${FORGE_BASE}/releases/download/${RELEASE_TAG}/SHA256SUMS
"

ARCHIVE_FILE="$TMP_DIR/cccp-archive.tar.gz"
CHECKSUM_FILE="$TMP_DIR/SHA256SUMS"

echo "★ Fetching static binary archive for ${CYAN}${ALIAS}${NC}..."

DOWNLOAD_SUCCESS=0
for url in $URLS; do
    [ -z "$url" ] && continue
    if curl -fsSL "$url" -o "$ARCHIVE_FILE" 2>/dev/null; then
        if [ -s "$ARCHIVE_FILE" ]; then
            echo "✔ Downloaded archive from ${url}"
            DOWNLOAD_SUCCESS=1
            break
        fi
    fi
done

if [ "$DOWNLOAD_SUCCESS" -ne 1 ]; then
    echo "${RED}Error: Failed to download cccp binary archive from any distribution mirror.${NC}" >&2
    echo "Please verify release availability at: ${FORGE_BASE}/releases" >&2
    exit 1
fi

# Attempt to download checksum file
for c_url in $CHECKSUM_URLS; do
    [ -z "$c_url" ] && continue
    if curl -fsSL "$c_url" -o "$CHECKSUM_FILE" 2>/dev/null; then
        if [ -s "$CHECKSUM_FILE" ]; then
            break
        fi
    fi
done

# -----------------------------------------------------------------------------
# 4. Integrity Validation (SHA-256)
# -----------------------------------------------------------------------------
if [ -f "$CHECKSUM_FILE" ] && [ -s "$CHECKSUM_FILE" ]; then
    EXPECTED_HASH=$(grep -E "(cccp.*${ALIAS}\.tar\.gz|cccp.*${TARGET_TRIPLE}\.tar\.gz)" "$CHECKSUM_FILE" | head -n 1 | awk '{print $1}' || true)
    if [ -n "$EXPECTED_HASH" ]; then
        ACTUAL_HASH=$(compute_sha256 "$ARCHIVE_FILE")
        if [ -n "$ACTUAL_HASH" ]; then
            if [ "$ACTUAL_HASH" != "$EXPECTED_HASH" ]; then
                echo "" >&2
                echo "${RED}★ ВНИМАНИЕ! САБОТАЖ! ★${NC}" >&2
                echo "${RED}Error: SHA-256 integrity verification failed!${NC}" >&2
                echo "Expected: ${EXPECTED_HASH}" >&2
                echo "Actual:   ${ACTUAL_HASH}" >&2
                exit 1
            else
                SHORT_HASH=$(printf "%.16s" "$ACTUAL_HASH")
                echo "✔ Integrity verified: SHA-256 matches Gosplan registry (${SHORT_HASH}...)."
            fi
        fi
    fi
fi

# -----------------------------------------------------------------------------
# 5. Extraction & Installation
# -----------------------------------------------------------------------------
EXTRACT_DIR="$TMP_DIR/extracted"
mkdir -p "$EXTRACT_DIR"
tar -xzf "$ARCHIVE_FILE" -C "$EXTRACT_DIR"

FOUND_BIN=$(find "$EXTRACT_DIR" -type f -name "cccp" | head -n 1 || true)
if [ -z "$FOUND_BIN" ] || [ ! -f "$FOUND_BIN" ]; then
    echo "${RED}Error: Executable binary 'cccp' not found inside downloaded archive.${NC}" >&2
    exit 1
fi

cp "$FOUND_BIN" "$INSTALL_DIR/cccp"
chmod +x "$INSTALL_DIR/cccp"

# -----------------------------------------------------------------------------
# 6. Verification and Smoke Test
# -----------------------------------------------------------------------------
if "$INSTALL_DIR/cccp" --version >/dev/null 2>&1; then
    INSTALLED_VER=$("$INSTALL_DIR/cccp" --version 2>/dev/null || echo "installed")
    echo "${GREEN}✔ CCCP ${INSTALLED_VER} successfully deployed into ${INSTALL_DIR}/cccp!${NC}"
else
    echo "${GREEN}✔ CCCP binary placed at ${INSTALL_DIR}/cccp.${NC}"
fi

# PATH check notification
case ":$PATH:" in
    *":$INSTALL_DIR:"*) ;;
    *)
        echo ""
        echo "${GOLD}Notice: $INSTALL_DIR is not yet in your PATH.${NC}"
        echo "Add it to your ~/.profile, ~/.bashrc or ~/.zshrc:"
        echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
        ;;
esac

echo ""
echo "★ To equip any Git repository, run:  ${CYAN}cd my-project && cccp install${NC}"
echo "★ To craft a compliant commit:      ${CYAN}cccp cz${NC}"
echo "${GOLD}★ Comrade, your workstation is now compliant!${NC}"
exit 0

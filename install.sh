#!/bin/sh
# ==============================================================================
# CCCP - Conventional Commits Compliance Program
# Automated Build & Installation Script
# «В СВЕТЛОЕ БУДУЩЕЕ С ЧИСТЫМ GIT!»
# ==============================================================================

set -eu

# Color output helpers (disable if NO_COLOR is set or stdout is not a tty)
if [ -t 1 ] && [ -z "${NO_COLOR:-}" ]; then
    BOLD="\033[1m"
    GREEN="\033[1;32m"
    YELLOW="\033[1;33m"
    RED="\033[1;31m"
    CYAN="\033[1;36m"
    RESET="\033[0m"
else
    BOLD=""
    GREEN=""
    YELLOW=""
    RED=""
    CYAN=""
    RESET=""
fi

show_help() {
    cat <<EOF
CCCP Build & Installation Inspectorate

Usage:
  ./install.sh [OPTIONS]

Options:
  --prefix <DIR>     Set base installation prefix (default: \$HOME/.local or /usr/local for root)
  --bin-dir <DIR>    Set specific directory for binary (default: <prefix>/bin)
  --dest-dir <DIR>   Set staging destination prefix (for packagers / DESTDIR)
  --target <TRIPLE>  Specify custom Cargo target triple
  --debug            Build debug profile instead of optimized release
  --dry-run          Simulate checks and commands without building or installing
  -h, --help         Display this help message

Environment Variables:
  PREFIX             Alternative to --prefix
  BINDIR             Alternative to --bin-dir
  DESTDIR            Alternative to --dest-dir
  TARGET             Alternative to --target

Examples:
  ./install.sh
  ./install.sh --prefix /usr/local
  ./install.sh --bin-dir /usr/bin
  DESTDIR=/tmp/pkg ./install.sh
EOF
}

# Defaults
MODE="release"
DRY_RUN=0
TARGET_TRIPLE="${TARGET:-}"
DEST_STAGING="${DESTDIR:-}"

# Default PREFIX based on user id
if [ -n "${PREFIX:-}" ]; then
    INSTALL_PREFIX="$PREFIX"
elif [ "$(id -u 2>/dev/null || echo 1000)" -eq 0 ]; then
    INSTALL_PREFIX="/usr/local"
else
    INSTALL_PREFIX="$HOME/.local"
fi

INSTALL_BIN_DIR="${BINDIR:-$INSTALL_PREFIX/bin}"

# Parse arguments
while [ $# -gt 0 ]; do
    case "$1" in
        --prefix)
            if [ -z "${2:-}" ]; then
                echo "${RED}Error: --prefix requires an argument.${RESET}" >&2
                exit 1
            fi
            INSTALL_PREFIX="$2"
            INSTALL_BIN_DIR="$INSTALL_PREFIX/bin"
            shift 2
            ;;
        --bin-dir)
            if [ -z "${2:-}" ]; then
                echo "${RED}Error: --bin-dir requires an argument.${RESET}" >&2
                exit 1
            fi
            INSTALL_BIN_DIR="$2"
            shift 2
            ;;
        --dest-dir)
            if [ -z "${2:-}" ]; then
                echo "${RED}Error: --dest-dir requires an argument.${RESET}" >&2
                exit 1
            fi
            DEST_STAGING="$2"
            shift 2
            ;;
        --target)
            if [ -z "${2:-}" ]; then
                echo "${RED}Error: --target requires an argument.${RESET}" >&2
                exit 1
            fi
            TARGET_TRIPLE="$2"
            shift 2
            ;;
        --debug)
            MODE="debug"
            shift
            ;;
        --dry-run)
            DRY_RUN=1
            shift
            ;;
        -h|--help)
            show_help
            exit 0
            ;;
        *)
            echo "${RED}Error: Unknown argument '$1'.${RESET}" >&2
            echo "Run './install.sh --help' for usage." >&2
            exit 1
            ;;
    esac
done

FINAL_DEST_DIR="${DEST_STAGING}${INSTALL_BIN_DIR}"

echo "${BOLD}======================================================================${RESET}"
echo "${BOLD} ★ CCCP State Production & Installation Inspectorate ★${RESET}"
echo "${BOLD}======================================================================${RESET}"
echo "Installation Profile: ${CYAN}${MODE}${RESET}"
echo "Target Bin Directory: ${CYAN}${INSTALL_BIN_DIR}${RESET}"
if [ -n "$DEST_STAGING" ]; then
    echo "Staging Prefix:       ${CYAN}${DEST_STAGING}${RESET}"
fi
if [ -n "$TARGET_TRIPLE" ]; then
    echo "Cargo Target:         ${CYAN}${TARGET_TRIPLE}${RESET}"
fi
echo

# 1. Verify Rust & Cargo toolchain
echo "${BOLD}[1/4] Inspecting Rust toolchain...${RESET}"
if ! command -v cargo >/dev/null 2>&1; then
    echo "${RED}Error: Cargo toolchain not found!${RESET}" >&2
    echo "The Soviet State requires Rust 1.75+ to forge CCCP binaries." >&2
    echo "Install Rust via official channels: https://rustup.rs" >&2
    exit 1
fi

CARGO_VER=$(cargo --version)
echo "${GREEN}  ✓ Found:${RESET} ${CARGO_VER}"

if [ "$DRY_RUN" -eq 1 ]; then
    echo "${YELLOW}[DRY-RUN] Would compile with:${RESET} cargo build"
    if [ "$MODE" = "release" ]; then
        echo "  Flags: --release"
    fi
    if [ -n "$TARGET_TRIPLE" ]; then
        echo "  Target: --target $TARGET_TRIPLE"
    fi
    echo "${YELLOW}[DRY-RUN] Would install binary to:${RESET} ${FINAL_DEST_DIR}/cccp"
    echo "${GREEN}✓ Dry-run completed successfully.${RESET}"
    exit 0
fi

# 2. Build binary
echo
echo "${BOLD}[2/4] Forging binary with Cargo...${RESET}"
BUILD_ARGS="build"
if [ "$MODE" = "release" ]; then
    BUILD_ARGS="$BUILD_ARGS --release"
fi
if [ -n "$TARGET_TRIPLE" ]; then
    BUILD_ARGS="$BUILD_ARGS --target $TARGET_TRIPLE"
fi

# Run cargo build
# shellcheck disable=SC2086
cargo $BUILD_ARGS

# 3. Locate compiled binary
echo
echo "${BOLD}[3/4] Locating forged artifact...${RESET}"
SOURCE_BIN=""

if [ -n "$TARGET_TRIPLE" ] && [ -f "target/${TARGET_TRIPLE}/${MODE}/cccp" ]; then
    SOURCE_BIN="target/${TARGET_TRIPLE}/${MODE}/cccp"
elif [ -f "target/x86_64-unknown-linux-musl/${MODE}/cccp" ]; then
    SOURCE_BIN="target/x86_64-unknown-linux-musl/${MODE}/cccp"
elif [ -f "target/${MODE}/cccp" ]; then
    SOURCE_BIN="target/${MODE}/cccp"
else
    # Fallback search in target directory
    CANDIDATE=$(find target -type f -name cccp -path "*/${MODE}/cccp" 2>/dev/null | head -n 1 || true)
    if [ -n "$CANDIDATE" ] && [ -f "$CANDIDATE" ]; then
        SOURCE_BIN="$CANDIDATE"
    fi
fi

if [ -z "$SOURCE_BIN" ] || [ ! -f "$SOURCE_BIN" ]; then
    echo "${RED}Error: Could not locate compiled binary in target/${RESET}" >&2
    exit 1
fi

echo "${GREEN}  ✓ Artifact found:${RESET} ${SOURCE_BIN}"

# 4. Install binary into destination
echo
echo "${BOLD}[4/4] Deploying to destination directory...${RESET}"
mkdir -p "$FINAL_DEST_DIR"

if command -v install >/dev/null 2>&1; then
    install -m 755 "$SOURCE_BIN" "${FINAL_DEST_DIR}/cccp"
else
    cp -f "$SOURCE_BIN" "${FINAL_DEST_DIR}/cccp"
    chmod 755 "${FINAL_DEST_DIR}/cccp"
fi

echo "${GREEN}  ✓ Deployed:${RESET} ${FINAL_DEST_DIR}/cccp"

# Verify execution if not cross-compiling or staged
if [ -z "$DEST_STAGING" ]; then
    INSTALLED_VER=$("${FINAL_DEST_DIR}/cccp" --version 2>/dev/null || echo "version-check-unavailable")
    echo "${GREEN}  ✓ Verified:${RESET} ${INSTALLED_VER}"
fi

# PATH inspection
echo
case ":${PATH}:" in
    *":${INSTALL_BIN_DIR}:"*)
        echo "${GREEN}★ Directory ${INSTALL_BIN_DIR} is already present in your PATH.${RESET}"
        ;;
    *)
        echo "${YELLOW}Notice: ${INSTALL_BIN_DIR} is not in your current PATH.${RESET}"
        echo "To run 'cccp' from anywhere, append the following line to your shell profile (~/.bashrc, ~/.zshrc):"
        echo "  ${CYAN}export PATH=\"\$PATH:${INSTALL_BIN_DIR}\"${RESET}"
        ;;
esac

echo
echo "${BOLD}======================================================================${RESET}"
echo "${GREEN} ★ CCCP successfully installed for the Motherland! ★${RESET}"
echo " Execute 'cccp --help' or 'cccp status' to begin inspection."
echo "${BOLD}======================================================================${RESET}"

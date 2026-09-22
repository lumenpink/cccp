#!/bin/sh

# =============================================================================
# CCCP — Conventional Commits Compliance Program
# Universal POSIX Shell Installer & Workstation Provisioner
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

# Parse channel or flag arguments
CHANNEL="stable"
for arg in "$@"; do
    case "$arg" in
        nightly|--nightly) CHANNEL="nightly" ;;
        stable|--stable)   CHANNEL="stable" ;;
    esac
done

INSTALL_DIR="${CCCP_INSTALL_DIR:-$HOME/.local/bin}"
mkdir -p "$INSTALL_DIR"

if [ "$CHANNEL" = "nightly" ]; then
    PRIMARY_URL="https://cccp.2lp.in/nightly/cccp.sh"
    FALLBACK_URL="https://codeberg.org/lumenpink/cccp/raw/branch/main/cccp.sh"
    echo "★ Channel selected: ${GOLD}NIGHTLY${NC} (rolling edge build)"
else
    PRIMARY_URL="https://cccp.2lp.in/cccp.sh"
    FALLBACK_URL="https://codeberg.org/lumenpink/cccp/raw/branch/main/cccp.sh"
    echo "★ Channel selected: ${GOLD}STABLE${NC} (official release)"
fi

echo "★ Fetching standalone binary into ${CYAN}$INSTALL_DIR/cccp${NC}..."

if curl -fsSL "$PRIMARY_URL" -o "$INSTALL_DIR/cccp" 2>/dev/null; then
    echo "✔ Downloaded from primary distribution (${PRIMARY_URL})."
elif curl -fsSL "$FALLBACK_URL" -o "$INSTALL_DIR/cccp" 2>/dev/null; then
    echo "✔ Downloaded from Codeberg fallback mirror."
else
    echo "${RED}Error: Failed to download cccp from distribution endpoints.${NC}" >&2
    exit 1
fi

chmod +x "$INSTALL_DIR/cccp"

# Verify execution
if "$INSTALL_DIR/cccp" --version >/dev/null 2>&1; then
    INSTALLED_VER=$("$INSTALL_DIR/cccp" --version 2>/dev/null || echo "installed")
    echo "${GREEN}✔ CCCP ${INSTALLED_VER} successfully deployed!${NC}"
else
    echo "${GREEN}✔ CCCP binary placed at $INSTALL_DIR/cccp.${NC}"
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

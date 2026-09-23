#!/bin/sh
# =============================================================================
# CCCP Shell Linting & Format Checker
# Runs shellcheck and shfmt against the POSIX shell source tree.
# Gracefully degrades if tools are absent (locally); CI installs them.
#
# Usage:
#   ./scripts/lint.sh              -- syntax + shellcheck + shfmt check
#   ./scripts/lint.sh --syntax     -- sh -n syntax check only
#   ./scripts/lint.sh --shellcheck -- shellcheck only (requires shellcheck)
#   ./scripts/lint.sh --fmt        -- shfmt format check only (requires shfmt)
#   ./scripts/lint.sh --fix        -- auto-format with shfmt (in-place)
# =============================================================================
set -eu

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
SRC_DIR="${REPO_ROOT}/src"

# shfmt flags: -i 4 (4-space indent), -ci (indent switch cases),
#              -bn (binary ops at start of line), -ln posix (POSIX dialect)
SHFMT_FLAGS="-i 4 -ci -bn -ln posix"

# shellcheck flags: POSIX dialect, exclude SC2039 (POSIX allowed in some contexts)
SHELLCHECK_FLAGS="-s sh"

# Colour codes (safe: only output if tty)
if [ -t 1 ]; then
    _RED='\033[0;31m'; _GRN='\033[0;32m'; _YLW='\033[1;33m'; _RST='\033[0m'
else
    _RED=''; _GRN=''; _YLW=''; _RST=''
fi

pass()  { printf "${_GRN}  [PASS]${_RST} %s\n" "$1"; }
fail()  { printf "${_RED}  [FAIL]${_RST} %s\n" "$1" >&2; }
warn()  { printf "${_YLW}  [WARN]${_RST} %s\n" "$1" >&2; }
info()  { printf "  %s\n" "$1"; }

# -----------------------------------------------------------------------------
# Collect target shell scripts
# -----------------------------------------------------------------------------
collect_scripts() {
    find "$SRC_DIR" -type f -name "*.sh" 2>/dev/null
    # Also include root-level scripts
    for _f in "${REPO_ROOT}/combine.sh" "${REPO_ROOT}/cccp.sh"; do
        [ -f "$_f" ] && printf '%s\n' "$_f"
    done
}

# -----------------------------------------------------------------------------
# Step 1: POSIX syntax check via sh -n (always runs, no external deps)
# -----------------------------------------------------------------------------
run_syntax_check() {
    printf 'Running sh -n syntax check...\n'
    _errors=0
    while read -r _f; do
        if sh -n "$_f" 2>/dev/null; then
            pass "$(basename "$_f")"
        else
            sh -n "$_f" 2>&1 | while read -r _line; do
                fail "$_line"
            done
            _errors=$(( _errors + 1 ))
        fi
    done <<EOF
$(collect_scripts)
EOF
    if [ "$_errors" -gt 0 ]; then
        printf '\n%b%d script(s) failed syntax check.%b\n' "$_RED" "$_errors" "$_RST" >&2
        return 1
    fi
    printf '%bAll scripts passed sh -n syntax check.%b\n' "$_GRN" "$_RST"
    return 0
}

# -----------------------------------------------------------------------------
# Step 2: shellcheck static analysis
# -----------------------------------------------------------------------------
run_shellcheck() {
    if ! command -v shellcheck >/dev/null 2>&1; then
        warn "shellcheck not found — skipping static analysis."
        warn "Install: https://github.com/koalaman/shellcheck#installing"
        return 0
    fi

    _version=$(shellcheck --version 2>/dev/null | grep 'version:' | cut -d' ' -f2 || echo "unknown")
    printf 'Running shellcheck %s...\n' "$_version"

    _errors=0
    while read -r _f; do
        # Skip combined bundle (it has intentional cross-file constructs)
        case "$_f" in */cccp.sh) continue ;; esac
        if shellcheck $SHELLCHECK_FLAGS "$_f" 2>&1; then
            pass "$(basename "$_f")"
        else
            _errors=$(( _errors + 1 ))
        fi
    done <<EOF
$(collect_scripts)
EOF
    if [ "$_errors" -gt 0 ]; then
        printf '\n%b%d script(s) failed shellcheck.%b\n' "$_RED" "$_errors" "$_RST" >&2
        return 1
    fi
    printf '%bAll scripts passed shellcheck.%b\n' "$_GRN" "$_RST"
    return 0
}

# -----------------------------------------------------------------------------
# Step 3: shfmt format check
# -----------------------------------------------------------------------------
run_shfmt_check() {
    if ! command -v shfmt >/dev/null 2>&1; then
        warn "shfmt not found — skipping format check."
        warn "Install: https://github.com/mvdan/sh#shfmt"
        return 0
    fi

    _version=$(shfmt --version 2>/dev/null || echo "unknown")
    printf 'Running shfmt %s (check mode)...\n' "$_version"

    # shfmt --diff exits 1 if files differ
    _errors=0
    while read -r _f; do
        case "$_f" in */cccp.sh) continue ;; esac
        # shellcheck disable=SC2086
        if shfmt $SHFMT_FLAGS --diff "$_f" >/dev/null 2>&1; then
            pass "$(basename "$_f")"
        else
            fail "$(basename "$_f") — format drift detected. Run: scripts/lint.sh --fix"
            _errors=$(( _errors + 1 ))
        fi
    done <<EOF
$(collect_scripts)
EOF
    if [ "$_errors" -gt 0 ]; then
        printf '\n%b%d script(s) require formatting. Run: scripts/lint.sh --fix%b\n' \
            "$_RED" "$_errors" "$_RST" >&2
        return 1
    fi
    printf '%bAll scripts are correctly formatted.%b\n' "$_GRN" "$_RST"
    return 0
}

# -----------------------------------------------------------------------------
# Step 4: shfmt auto-fix (in-place)
# -----------------------------------------------------------------------------
run_shfmt_fix() {
    if ! command -v shfmt >/dev/null 2>&1; then
        printf '%bError: shfmt not found. Install: https://github.com/mvdan/sh#shfmt%b\n' \
            "$_RED" "$_RST" >&2
        return 1
    fi

    _version=$(shfmt --version 2>/dev/null || echo "unknown")
    printf 'Running shfmt %s (write mode)...\n' "$_version"

    while read -r _f; do
        case "$_f" in */cccp.sh) continue ;; esac
        # shellcheck disable=SC2086
        if shfmt $SHFMT_FLAGS --write "$_f" 2>&1; then
            info "Formatted: $(basename "$_f")"
        else
            fail "Failed to format: $(basename "$_f")"
        fi
    done <<EOF
$(collect_scripts)
EOF
    printf '%bFormatting complete. Review changes before committing.%b\n' "$_GRN" "$_RST"
}

# -----------------------------------------------------------------------------
# Help
# -----------------------------------------------------------------------------
show_help() {
    cat <<EOF
CCCP Shell Linter & Formatter
Usage: $(basename "$0") [mode]

Modes:
  (none)         Run all checks: syntax + shellcheck + shfmt diff
  --syntax       sh -n syntax check only (no external deps)
  --shellcheck   shellcheck static analysis only (requires shellcheck)
  --fmt          shfmt format compliance check only (requires shfmt)
  --fix          Auto-format all src/ scripts with shfmt (requires shfmt)
  -h, --help     Show this help message

Tool configuration:
  shellcheck flags: ${SHELLCHECK_FLAGS}
  shfmt flags:      ${SHFMT_FLAGS}

Install missing tools:
  shellcheck: https://github.com/koalaman/shellcheck#installing
  shfmt:      https://github.com/mvdan/sh#shfmt
EOF
}

# -----------------------------------------------------------------------------
# Main
# -----------------------------------------------------------------------------
main() {
    _mode="all"

    case "${1:-}" in
        --syntax)     _mode="syntax" ;;
        --shellcheck) _mode="shellcheck" ;;
        --fmt)        _mode="fmt" ;;
        --fix)        _mode="fix" ;;
        -h|--help)    show_help; exit 0 ;;
        "")           _mode="all" ;;
        *)
            printf 'Unknown mode: %s\n' "$1" >&2
            show_help >&2
            exit 1
            ;;
    esac

    _rc=0
    case "$_mode" in
        syntax)
            run_syntax_check || _rc=1
            ;;
        shellcheck)
            run_shellcheck || _rc=1
            ;;
        fmt)
            run_shfmt_check || _rc=1
            ;;
        fix)
            run_shfmt_fix || _rc=1
            ;;
        all)
            printf '=== CCCP Shell Lint & Format Check ===\n\n'
            printf '[1/3] Syntax Check\n'
            run_syntax_check || _rc=1
            printf '\n[2/3] Static Analysis (shellcheck)\n'
            run_shellcheck || _rc=1
            printf '\n[3/3] Format Compliance (shfmt)\n'
            run_shfmt_check || _rc=1
            if [ "$_rc" -eq 0 ]; then
                printf '\n%bAll checks passed.%b\n' "$_GRN" "$_RST"
            else
                printf '\n%bOne or more checks failed.%b\n' "$_RED" "$_RST" >&2
            fi
            ;;
    esac

    exit "$_rc"
}

main "$@"

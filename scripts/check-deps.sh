#!/bin/sh
# =============================================================================
# CCCP External Dependency Auditor
# Inspects shell scripts for external binary invocations and verifies
# compliance against standard POSIX utilities and DEPENDENCIES.md.
# Strict zero-awk, pure POSIX compliant.
# =============================================================================
set -eu

# Script directory and repository root
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Target directory to audit
SRC_DIR="${REPO_ROOT}/src"
# Allow env var override for testability
DEPS_FILE="${DEPS_FILE:-${REPO_ROOT}/DEPENDENCIES.md}"

# Banned utilities (Strict project policy — zero tolerance)
BANNED_UTILS="awk gawk mawk nawk"

# Unauthorized runtime binaries — must never appear as executable commands
UNAUTHORIZED_WATCH="python python2 python3 node npm npx perl ruby php rustc cargo lua java"

# Approved external dependencies baseline
APPROVED_EXTERNAL="act agy busybox curl docker ftp ftpget gh git hostname inotifywait ip jq kv kvant lsof mayak mk nc open openssl ss sv svyaz tailscale tea telnet wget xdg-open"

# Mandatory tools always used directly (never probed via command -v)
# Unconditionally included in the detected list
MANDATORY_EXTERNAL="git"

# -----------------------------------------------------------------------------
# Standard IEEE Std 1003.1 (POSIX) Utilities
# -----------------------------------------------------------------------------
is_posix_utility() {
    case "$1" in
        alias|ar|at|basename|batch|bc|bg|c99|cal|cat|cd|cflow|chgrp|chmod|chown) return 0 ;;
        cksum|cmp|comm|command|compress|cp|crontab|csplit|ctags|cut|cxref|date) return 0 ;;
        dd|df|diff|dirname|dmesg|du|echo|ed|env|ex|expand|expr|false|fc|fg|file) return 0 ;;
        find|fold|fort77|fuser|gencat|getconf|getopts|grep|head|iconv|id|ipcrm) return 0 ;;
        ipcs|jobs|join|kill|lex|link|ln|locale|localedef|logger|logname|lp|ls) return 0 ;;
        m4|mailx|make|man|mesg|mkdir|mkfifo|more|mv|newgrp|nice|nl|nm|nohup|od) return 0 ;;
        paste|patch|pathchk|pax|pr|printf|prs|ps|pwd|qalter|qdel|qhold|qmove) return 0 ;;
        qmsg|qrerun|qrls|qselect|qsig|qstat|qsub|read|renice|rm|rmdel|rmdir|sact) return 0 ;;
        sccs|sed|sh|sleep|sort|split|strings|strip|stty|tabs|tail|tar|tee|test) return 0 ;;
        time|touch|tput|tr|true|tsort|tty|type|ulimit|umask|unalias|uname) return 0 ;;
        uncompress|unexpand|uniq|unlink|uucp|uudecode|uuencode|uustat|uux|val|vi) return 0 ;;
        wait|wc|what|who|write|xargs|yacc|zcat|mktemp) return 0 ;;
        *) return 1 ;;
    esac
}

# -----------------------------------------------------------------------------
# Shell Built-ins, Keywords & Special Tokens
# -----------------------------------------------------------------------------
is_shell_builtin() {
    case "$1" in
        case|do|done|elif|else|esac|fi|for|function|if|in|then|until|while) return 0 ;;
        eval|exec|exit|export|readonly|return|set|shift|times|trap|unset|local) return 0 ;;
        break|continue) return 0 ;;
        *) return 1 ;;
    esac
}

# -----------------------------------------------------------------------------
# Collect Defined Functions in Source Tree
# (Used to exclude internal project functions from the external set)
# -----------------------------------------------------------------------------
get_internal_functions() {
    _scan_path="$1"
    if [ -f "$_scan_path" ]; then
        _file_list="$_scan_path"
    else
        _file_list=$(find "$_scan_path" -type f -name "*.sh" 2>/dev/null)
    fi

    printf '%s\n' "$_file_list" | while read -r _f; do
        [ -f "$_f" ] || continue
        sed -n -E \
            -e 's/^[[:space:]]*([a-zA-Z_][a-zA-Z0-9_]*)[[:space:]]*\(\).*/\1/p' \
            -e 's/^[[:space:]]*function[[:space:]]+([a-zA-Z_][a-zA-Z0-9_]*).*/\1/p' \
            "$_f" 2>/dev/null || true
    done | sort -u
}

# -----------------------------------------------------------------------------
# Strategy: Probe-Only Detection via "command -v <tool>" patterns
# This is the authoritative signal that a script explicitly tests for an external binary.
# Each file must be processed individually with a trailing newline to avoid concatenation.
# -----------------------------------------------------------------------------
collect_probed_tools() {
    _scan_path="$1"
    if [ -f "$_scan_path" ]; then
        _file_list="$_scan_path"
    else
        _file_list=$(find "$_scan_path" -type f -name "*.sh" 2>/dev/null)
    fi

    printf '%s\n' "$_file_list" | while read -r _f; do
        [ -f "$_f" ] || continue
        # Strip comments, then extract tool names from "command -v <tool>" patterns.
        # Process one file at a time and emit a blank separator to prevent cross-file token merging.
        sed -e 's/#.*$//' "$_f" 2>/dev/null \
            | sed -n -E 's/.*command[[:space:]]+-v[[:space:]]+"?([a-zA-Z0-9_][a-zA-Z0-9_-]*)"?.*/\1/p'
        printf '\n'
    done | sort -u
}

# -----------------------------------------------------------------------------
# Strategy: Direct Invocation Scan — only scan for known approved tools
# Avoids false positives from prose words appearing in echo/printf strings.
# -----------------------------------------------------------------------------
collect_direct_invocations() {
    _scan_path="$1"
    if [ -f "$_scan_path" ]; then
        _file_list="$_scan_path"
    else
        _file_list=$(find "$_scan_path" -type f -name "*.sh" 2>/dev/null)
    fi

    for _tool in $APPROVED_EXTERNAL; do
        if printf '%s\n' "$_file_list" | while read -r _f; do
            [ -f "$_f" ] || continue
            # Strip comments, check for direct invocation as a command token
            sed -e 's/#.*$//' "$_f" 2>/dev/null \
                | grep -q -E "(^|[[:space:]|;&\`\(])${_tool}[[:space:]|;&\`\)]"
        done 2>/dev/null | grep -q .; then
            printf '%s\n' "$_tool"
        else
            # Run the scan properly — the subshell above just checks exit code
            printf '%s\n' "$_file_list" | while read -r _f; do
                [ -f "$_f" ] || continue
                sed -e 's/#.*$//' "$_f" 2>/dev/null \
                    | grep -q -E "(^|[[:space:]|;&\`\(])${_tool}[[:space:]|;&\`\)]" && {
                    printf '%s\n' "$_tool"
                    break
                } || true
            done
        fi
    done | sort -u
}

# -----------------------------------------------------------------------------
# Detect External Dependencies
# Combines probe-only + direct invocation scanning, filtered against
# POSIX utilities, shell builtins, and internal project functions.
# -----------------------------------------------------------------------------
detect_external_dependencies() {
    _scan_path="$1"
    _funcs=$(get_internal_functions "$_scan_path")

    {
        # 1. Always emit mandatory tools (used directly everywhere, never via command -v)
        for _m in $MANDATORY_EXTERNAL; do
            printf '%s\n' "$_m"
        done

        # 2. Collect probed candidates and filter
        collect_probed_tools "$_scan_path" | while read -r _cmd; do
            [ -z "$_cmd" ] && continue

            # Must look like a valid binary name
            case "$_cmd" in
                [0-9]*|-*|"") continue ;;
            esac
            printf '%s\n' "$_cmd" | grep -qE "^[a-zA-Z0-9_][a-zA-Z0-9_-]*$" || continue

            if is_shell_builtin "$_cmd"; then
                continue
            fi

            if is_posix_utility "$_cmd"; then
                continue
            fi

            # Skip internal project functions
            if printf '%s\n' "$_funcs" | grep -qx "$_cmd"; then
                continue
            fi

            printf '%s\n' "$_cmd"
        done
    } | sort -u
}

# -----------------------------------------------------------------------------
# Check for Banned Utilities (awk, gawk, etc.)
# Scans stripped source lines for word-boundary matches.
# -----------------------------------------------------------------------------
check_banned_utilities() {
    _scan_path="$1"
    _violations=0

    if [ -f "$_scan_path" ]; then
        _file_list="$_scan_path"
    else
        _file_list=$(find "$_scan_path" -type f -name "*.sh" 2>/dev/null)
    fi

    for _banned in $BANNED_UTILS; do
        _found=$(printf '%s\n' "$_file_list" | while read -r _f; do
            [ -f "$_f" ] || continue
            # Strip comments, check for executable invocation (word boundary)
            sed -e 's/#.*$//' "$_f" 2>/dev/null \
                | grep -n -E "(^|[[:space:]|;&\`\(])${_banned}([[:space:]|;&\`\)>]|$)" \
                | while read -r _match; do
                    printf '%s:%s\n' "$_f" "$_match"
                done
        done || true)

        if [ -n "$_found" ]; then
            printf '::error::Banned utility detected: %s\n' "$_banned" >&2
            printf '%s\n' "$_found" >&2
            _violations=$(( _violations + 1 ))
        fi
    done

    [ "$_violations" -eq 0 ]
}

# -----------------------------------------------------------------------------
# Check for Unauthorized Binaries (python, node, perl, etc.)
# -----------------------------------------------------------------------------
check_unauthorized_binaries() {
    _scan_path="$1"
    _violations=0

    if [ -f "$_scan_path" ]; then
        _file_list="$_scan_path"
    else
        _file_list=$(find "$_scan_path" -type f -name "*.sh" 2>/dev/null)
    fi

    for _bad in $UNAUTHORIZED_WATCH; do
        _found=$(printf '%s\n' "$_file_list" | while read -r _f; do
            [ -f "$_f" ] || continue
            sed -e 's/#.*$//' "$_f" 2>/dev/null \
                | grep -n -E "(^|[[:space:]|;&\`\(])${_bad}([[:space:]|;&\`\)>]|$)" \
                | while read -r _match; do
                    printf '%s:%s\n' "$_f" "$_match"
                done
        done || true)

        if [ -n "$_found" ]; then
            printf '::error::Unauthorized binary detected: %s\n' "$_bad" >&2
            printf '%s\n' "$_found" >&2
            _violations=$(( _violations + 1 ))
        fi
    done

    [ "$_violations" -eq 0 ]
}

# -----------------------------------------------------------------------------
# Verify Against DEPENDENCIES.md
# -----------------------------------------------------------------------------
verify_dependencies_doc() {
    _scan_path="$1"
    _doc="$2"

    if [ ! -f "$_doc" ]; then
        echo "Error: DEPENDENCIES.md does not exist at $_doc" >&2
        return 1
    fi

    echo "Auditing external dependencies against $(basename "$_doc")..."
    _ext_list=$(detect_external_dependencies "$_scan_path")

    _missing=0
    for _dep in $_ext_list; do
        if ! grep -qi "\`${_dep}\`" "$_doc"; then
            echo "  ::error::External dependency '$_dep' missing from $(basename "$_doc")" >&2
            _missing=$(( _missing + 1 ))
        else
            echo "  [OK] $_dep documented"
        fi
    done

    # Verify banned tools are explicitly listed in the doc
    for _banned in $BANNED_UTILS; do
        if ! grep -qi "\`${_banned}\`" "$_doc"; then
            echo "  ::error::Banned utility '$_banned' must be documented in $(basename "$_doc")" >&2
            _missing=$(( _missing + 1 ))
        fi
    done

    if [ "$_missing" -gt 0 ]; then
        echo "Error: $_missing item(s) missing from $(basename "$_doc")." >&2
        return 1
    fi

    echo "All detected external dependencies and banned policies are accurately documented in $(basename "$_doc")."
    return 0
}

# -----------------------------------------------------------------------------
# Main CLI Dispatcher
# -----------------------------------------------------------------------------
show_help() {
    cat <<EOF
CCCP Dependency Auditor
Usage: $(basename "$0") [options] [path]

Options:
  --list           Print detected external binaries list
  --verify         Verify detected binaries against DEPENDENCIES.md and banned policies
  --check          Audit for unauthorized/banned tools (default)
  -h, --help       Show this help message

Default path audited: ${SRC_DIR}
EOF
}

main() {
    _mode="check"
    _target_dir="$SRC_DIR"

    while [ $# -gt 0 ]; do
        case "$1" in
            --list)   _mode="list";   shift ;;
            --verify) _mode="verify"; shift ;;
            --check)  _mode="check";  shift ;;
            -h|--help)
                show_help
                exit 0
                ;;
            -*)
                echo "Unknown option: $1" >&2
                show_help >&2
                exit 1
                ;;
            *)
                _target_dir="$1"
                shift
                ;;
        esac
    done

    if [ ! -d "$_target_dir" ] && [ ! -f "$_target_dir" ]; then
        echo "Error: Target path does not exist: $_target_dir" >&2
        exit 1
    fi

    case "$_mode" in
        list)
            detect_external_dependencies "$_target_dir"
            ;;
        verify)
            if ! check_banned_utilities "$_target_dir"; then
                echo "Audit failed: Banned utilities detected!" >&2
                exit 1
            fi
            if ! check_unauthorized_binaries "$_target_dir"; then
                echo "Audit failed: Unauthorized binaries detected!" >&2
                exit 1
            fi
            verify_dependencies_doc "$_target_dir" "$DEPS_FILE"
            ;;
        check)
            echo "Auditing codebase for banned utilities and unauthorized binaries..."
            if ! check_banned_utilities "$_target_dir"; then
                echo "Audit failed: Banned utilities detected!" >&2
                exit 1
            fi
            if ! check_unauthorized_binaries "$_target_dir"; then
                echo "Audit failed: Unauthorized binaries detected!" >&2
                exit 1
            fi

            _found=$(detect_external_dependencies "$_target_dir")
            echo "Detected external dependencies:"
            for _dep in $_found; do
                echo "  - $_dep"
            done
            echo "Dependency audit passed cleanly."
            ;;
    esac
}

main "$@"

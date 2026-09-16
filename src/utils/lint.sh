#!/bin/sh

# Enable error handling
set -eu

# -----------------------------------------------------------------------------
# Lint Git commit history against Conventional Commits specification
# -----------------------------------------------------------------------------
lint_commits() {
    # Verify git repository
    if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
        echo "Error: Not a git repository" >&2
        return 1
    fi

    # Ensure config hierarchy is loaded
    if command -v load_hierarchical_config >/dev/null 2>&1; then
        load_hierarchical_config
    fi

    range="${1:-}"

    # Determine default range if none provided
    if [ -z "$range" ]; then
        if [ -n "${GITHUB_BASE_REF:-}" ]; then
            range="origin/${GITHUB_BASE_REF}..HEAD"
        elif git rev-parse --verify '@{u}' >/dev/null 2>&1; then
            range="@{u}..HEAD"
        elif git rev-parse --verify 'origin/main' >/dev/null 2>&1; then
            range="origin/main..HEAD"
        elif git rev-parse --verify 'HEAD~1' >/dev/null 2>&1; then
            range="HEAD~1..HEAD"
        else
            range="HEAD"
        fi
    fi

    # Verify that the range is valid in git
    commit_hashes=$(git rev-list --reverse "$range" 2>/dev/null || true)
    if [ -z "$commit_hashes" ]; then
        if git rev-parse "$range" >/dev/null 2>&1 || echo "$range" | grep -q '\.\.'; then
            echo "========================================================"
            echo " ★ CCCP Party Line Compliance (Commit Linting) ★"
            echo "========================================================"
            echo " Inspected Range:  $range"
            echo " Result:           0 commits found in range to inspect."
            echo " Status:           All clear for the State Plan."
            echo "========================================================"
            return 0
        else
            echo "Error: Invalid commit range or revision '$range'" >&2
            return 1
        fi
    fi

    echo "========================================================"
    echo " ★ CCCP Party Line Compliance (Commit Linting) ★"
    echo "========================================================"
    echo " Inspected Range:  $range"
    echo "========================================================"

    total_count=0
    passed_count=0
    failed_count=0

    for hash in $commit_hashes; do
        total_count=$((total_count + 1))
        short_hash=$(echo "$hash" | cut -c1-7)
        subject=$(git log -1 --format="%s" "$hash")

        # Capture validation output
        val_output=$(validate_commit_message "$subject" 2>&1 || true)
        val_status=$?

        if echo "$val_output" | grep -q "^Error:"; then
            val_status=1
        fi

        if [ $val_status -eq 0 ]; then
            passed_count=$((passed_count + 1))
            echo "  ✓ [$short_hash] $subject"
        else
            failed_count=$((failed_count + 1))
            echo "  ✗ [$short_hash] $subject"
            err_msg=$(echo "$val_output" | grep "^Error:" | head -n 1 || echo "Error: Invalid conventional commit message")
            echo "    ↳ $err_msg"
        fi
    done

    echo "========================================================"
    echo " Inspection Summary: $total_count inspected, $passed_count compliant, $failed_count ideological deviations."

    if [ "$failed_count" -gt 0 ]; then
        echo " Directive: Order No. 227 - Not One Step Back from Clean Commits!"
        echo " FAILED: Non-compliant commits detected. Correct history before merge."
        echo "========================================================"
        return 1
    else
        echo " PASSED: Full ideological compliance verified. Glory to the collective!"
        echo "========================================================"
        return 0
    fi
}

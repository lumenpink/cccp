#!/bin/sh

# Enable error handling
set -eu

# -----------------------------------------------------------------------------
# Interactive Commit Wizard (CCCP Assembly Line / cz)
# -----------------------------------------------------------------------------
interactive_commit() {
    # Verify git repository
    if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
        echo "Error: Not a git repository" >&2
        return 1
    fi

    # Ensure config hierarchy is loaded
    if command -v load_hierarchical_config >/dev/null 2>&1; then
        load_hierarchical_config
    fi

    # Check if there are staged changes to commit
    staged_count=$(git diff --cached --name-only 2>/dev/null | wc -l | tr -d ' ')
    if [ "$staged_count" -eq 0 ]; then
        echo "Warning: No changes staged for commit." >&2
        echo "Stage your changes first with 'git add <files>' before assembling a commit." >&2
        return 1
    fi

    echo "========================================================"
    echo " ★ CCCP Assembly Line (Interactive Soviet Wizard) ★"
    echo "========================================================"
    echo " Step 1: Select the type of change"
    echo "--------------------------------------------------------"

    # List available types
    types_list="${COMMIT_TYPES:-feat fix perf refactor revert chore build ci docs ops style test merge}"
    type_idx=1
    for t in $types_list; do
        desc=""
        if command -v get_type_description >/dev/null 2>&1; then
            desc=$(get_type_description "$t")
        fi
        printf "  %2d) %-10s - %s\n" "$type_idx" "$t" "$desc"
        type_idx=$((type_idx + 1))
    done

    # Read type selection
    selected_type=""
    while [ -z "$selected_type" ]; do
        printf "\nSelect type [1-%d or name]: " "$((type_idx - 1))"
        read -r choice || return 1
        choice=$(echo "$choice" | tr -d '[:space:]')
        if echo "$choice" | grep -qE '^[0-9]+$'; then
            curr=1
            for t in $types_list; do
                if [ "$curr" -eq "$choice" ]; then
                    selected_type="$t"
                    break
                fi
                curr=$((curr + 1))
            done
        else
            for t in $types_list; do
                if [ "$choice" = "$t" ]; then
                    selected_type="$t"
                    break
                fi
            done
        fi
        if [ -z "$selected_type" ]; then
            echo "Invalid selection. Please choose a number between 1 and $((type_idx - 1)) or enter a type name."
        fi
    done

    # Step 2: Select or enter scope
    echo ""
    echo "--------------------------------------------------------"
    echo " Step 2: Scope of the change (optional)"
    echo "--------------------------------------------------------"
    echo " Common scopes: core, cli, config, ui, api, auth, db, docs, test, build"
    printf "Enter scope (press enter to skip): "
    read -r selected_scope || return 1
    selected_scope=$(echo "$selected_scope" | tr -d '[:space:]')

    # Step 3: Short subject line
    echo ""
    echo "--------------------------------------------------------"
    echo " Step 3: Commit subject (short imperative summary)"
    echo "--------------------------------------------------------"
    selected_subject=""
    while [ -z "$selected_subject" ]; do
        printf "Enter subject: "
        read -r selected_subject || return 1
        selected_subject=$(echo "$selected_subject" | sed -E 's/^[[:space:]]+|[[:space:]]+$//g')
        if [ -z "$selected_subject" ]; then
            echo "Commit subject cannot be empty."
        fi
    done

    # Step 4: Breaking change
    echo ""
    echo "--------------------------------------------------------"
    echo " Step 4: Breaking changes"
    echo "--------------------------------------------------------"
    printf "Does this change introduce a BREAKING CHANGE? (y/N): "
    read -r is_breaking || return 1
    is_breaking=$(echo "$is_breaking" | tr '[:upper:]' '[:lower:]')

    # Step 5: Longer description body
    echo ""
    echo "--------------------------------------------------------"
    echo " Step 5: Extended commit body (optional)"
    echo "--------------------------------------------------------"
    printf "Enter longer description (press enter to skip): "
    read -r selected_body || return 1
    selected_body=$(echo "$selected_body" | sed -E 's/^[[:space:]]+|[[:space:]]+$//g')

    # Assemble header
    header_prefix="$selected_type"
    if [ -n "$selected_scope" ]; then
        header_prefix="${header_prefix}(${selected_scope})"
    fi
    if [ "$is_breaking" = "y" ] || [ "$is_breaking" = "yes" ]; then
        header_prefix="${header_prefix}!"
    fi
    commit_header="${header_prefix}: ${selected_subject}"

    # Assemble full message
    full_message="$commit_header"
    if [ -n "$selected_body" ]; then
        full_message="${full_message}

${selected_body}"
    fi

    # Step 6: Confirmation
    echo ""
    echo "========================================================"
    echo " Proposed State Commit:"
    echo "--------------------------------------------------------"
    echo "$full_message"
    echo "--------------------------------------------------------"
    printf "Proceed with commit to the State archives? [Y/n]: "
    read -r confirm || return 1
    confirm=$(echo "$confirm" | tr '[:upper:]' '[:lower:]')

    if [ "$confirm" = "n" ] || [ "$confirm" = "no" ]; then
        echo "Commit aborted by worker directive."
        return 1
    fi

    # Perform commit
    if ! validate_commit_message "$full_message"; then
        echo "Error: Validation failed on constructed message." >&2
        return 1
    fi

    git commit -m "$full_message"
    echo "========================================================"
    echo " ★ Commit recorded into the State archives successfully! ★"
    echo "========================================================"
    return 0
}

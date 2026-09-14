#!/bin/sh

# Enable error handling
set -eu

# Find the git root directory
GIT_ROOT="$(git rev-parse --show-toplevel 2>/dev/null)"
if [ -z "$GIT_ROOT" ]; then
    echo "Error: Not a git repository"
    exit 1
fi

# Set up paths relative to git root
GIT_HOOKS_DIR="$GIT_ROOT/.git/hooks"

# Source the configuration
. "$GIT_ROOT/src/config/config.sh"

# -----------------------------------------------------------------------------
# Verify system prerequisites
# -----------------------------------------------------------------------------
check_prerequisites() {
    missing_tools=""
    for tool in git sed grep date cut tr; do
        if ! command -v "$tool" >/dev/null 2>&1; then
            missing_tools="$missing_tools $tool"
        fi
    done

    if [ -n "$missing_tools" ]; then
        echo "Error: Required system tools are missing from PATH:$missing_tools" >&2
        echo "Please install the missing tools and ensure they are accessible in your PATH." >&2
        return 1
    fi

    # Verify that we are inside a Git repository
    if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
        echo "Error: Not a git repository. This command must be executed within a valid Git repository." >&2
        return 1
    fi

    return 0
}

# -----------------------------------------------------------------------------
# Validate commit message format
# -----------------------------------------------------------------------------
validate_commit_message() {
    commit_msg="$1"
    type=""
    scope_part=""
    subject=""
    
    # Check for empty commit message
    if [ -z "$commit_msg" ]; then
        echo "Error: Commit message can't be empty"
        return 1
    fi

    # Check if header contains a colon
    if ! echo "$commit_msg" | grep -q ":"; then
        echo "Error: Commit message must follow format '<type>(<scope>): <subject>' or '<type>: <subject>'"
        return 1
    fi

    # Extract header line before colon
    header_prefix=$(echo "$commit_msg" | sed -E 's/:.*$//')
    subject=$(echo "$commit_msg" | sed -E 's/^[^:]*:[[:space:]]*//')

    # Detect breaking change marker '!'
    is_breaking=0
    if echo "$header_prefix" | grep -q '!$'; then
        is_breaking=1
        header_prefix="${header_prefix%!}"
    fi

    # Check for empty parentheses e.g. feat():
    if echo "$header_prefix" | grep -Fq '()'; then
        echo "Error: Scope cannot be empty"
        return 1
    fi

    # Extract type and scope
    if echo "$header_prefix" | grep -q "^[^(]*([^)]*)$"; then
        type=$(echo "$header_prefix" | sed -E 's/^([^(]+)\(([^)]*)\)$/\1/')
        scope_part=$(echo "$header_prefix" | sed -E 's/^([^(]+)\(([^)]*)\)$/\2/')
    else
        type="$header_prefix"
        scope_part=""
    fi
    
    # Clean up subject
    subject=$(echo "$subject" | sed -E 's/^[[:space:]]+//')
    
    # Validate type
    valid_type=0
    for t in $COMMIT_TYPES; do
        if [ "$type" = "$t" ]; then
            valid_type=1
            break
        fi
    done
    
    if [ $valid_type -eq 0 ]; then
        echo "Error: Invalid type '$type'. Must be one of: $COMMIT_TYPES"
        return 1
    fi
    
    if [ -z "$subject" ]; then
        echo "Error: Commit message must have a subject"
        return 1
    fi

    if [ -z "$scope_part" ]; then
        return 0
    fi
    
    # Validate scopes
    OLD_IFS="$IFS"
    IFS=','
    scope_count=0
    for scope_item in $scope_part; do
        scope_count=$((scope_count + 1))
        # Trim leading and trailing whitespace
        scope_item=$(echo "$scope_item" | sed -E 's/^[[:space:]]+|[[:space:]]+$//g')
        
        if echo "$scope_item" | grep -q "/"; then
            if [ "$DISABLE_SUBSCOPES" = "1" ]; then
                echo "Error: Subscopes are disabled"
                IFS="$OLD_IFS"
                return 1
            fi
            
            scope=$(echo "$scope_item" | cut -d'/' -f1 | sed -E 's/^[[:space:]]+|[[:space:]]+$//g')
            subscope=$(echo "$scope_item" | cut -d'/' -f2 | sed -E 's/^[[:space:]]+|[[:space:]]+$//g')
            
            # Validate scope
            valid_scope=0
            if [ "$ALLOW_ANY_SCOPE" = "1" ]; then
                valid_scope=1
            else
                IFS=" "
                for s in $COMMIT_SCOPES; do
                    if [ "$scope" = "$s" ]; then
                        valid_scope=1
                        break
                    fi
                done
                IFS=","
            fi
            
            if [ $valid_scope -eq 0 ]; then
                echo "Error: Invalid scope '$scope'. Must be one of: $COMMIT_SCOPES"
                IFS="$OLD_IFS"
                return 1
            fi
            
            # Validate subscope
            valid_subscope=0
            if [ "$ALLOW_ANY_SUBSCOPE" = "1" ]; then
                valid_subscope=1
            else
                IFS=" "
                for ss in $COMMIT_SUBSCOPES; do
                    if [ "$subscope" = "$ss" ]; then
                        valid_subscope=1
                        break
                    fi
                done
                IFS=","
            fi
            
            if [ $valid_subscope -eq 0 ]; then
                echo "Error: Invalid subscope '$subscope'. Must be one of: $COMMIT_SUBSCOPES"
                IFS="$OLD_IFS"
                return 1
            fi
        else
            scope=$(echo "$scope_item" | sed -E 's/^[[:space:]]+|[[:space:]]+$//g')
            
            valid_scope=0
            if [ "$ALLOW_ANY_SCOPE" = "1" ]; then
                valid_scope=1
            else
                IFS=" "
                for s in $COMMIT_SCOPES; do
                    if [ "$scope" = "$s" ]; then
                        valid_scope=1
                        break
                    fi
                done
                IFS=","
            fi
            
            if [ $valid_scope -eq 0 ]; then
                echo "Error: Invalid scope '$scope'. Must be one of: $COMMIT_SCOPES"
                IFS="$OLD_IFS"
                return 1
            fi
        fi
    done
    
    if [ $scope_count -gt 1 ] && [ "$DISABLE_MULTIPLE_SCOPES" = "1" ]; then
        echo "Error: Multiple scopes are disabled"
        IFS="$OLD_IFS"
        return 1
    fi
    
    IFS="$OLD_IFS"
    return 0
} 
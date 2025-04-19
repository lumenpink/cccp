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
    
    # Extract type, scope, and subject from commit message
    if echo "$commit_msg" | grep -q "^[^:]*([^)]*):"; then
        type=$(echo "$commit_msg" | sed -E 's/^([^(]+)\(([^)]*)\):(.*)$/\1/')
        scope_part=$(echo "$commit_msg" | sed -E 's/^([^(]+)\(([^)]*)\):(.*)$/\2/')
        subject=$(echo "$commit_msg" | sed -E 's/^([^(]+)\(([^)]*)\):(.*)$/\3/')
    else
        type=$(echo "$commit_msg" | sed -E 's/^([^:]*):(.*)$/\1/')
        scope_part=""
        subject=$(echo "$commit_msg" | sed -E 's/^([^:]*):(.*)$/\2/')
    fi
    
    # Clean up subject
    subject=$(echo "$subject" | sed -E 's/^[ ]+//')
    
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
    
    # Check for empty parentheses
    if echo "$commit_msg" | grep -q "^[^:]*():"; then
        echo "Error: Scope cannot be empty"
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
        
        if echo "$scope_item" | grep -q "/"; then
            if [ "$DISABLE_SUBSCOPES" = "1" ]; then
                echo "Error: Subscopes are disabled"
                return 1
            fi
            
            scope=$(echo "$scope_item" | cut -d'/' -f1)
            subscope=$(echo "$scope_item" | cut -d'/' -f2)
            
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
                return 1
            fi
        else
            scope=$scope_item
            
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
                return 1
            fi
        fi
    done
    
    if [ $scope_count -gt 1 ] && [ "$DISABLE_MULTIPLE_SCOPES" = "1" ]; then
        echo "Error: Multiple scopes are disabled"
        return 1
    fi
    
    IFS="$OLD_IFS"
    return 0
} 
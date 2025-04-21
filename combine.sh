#!/bin/sh

# =============================================================================
# Script to combine all modular files into a single file
# =============================================================================

# Get the script's directory
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
OUTPUT_FILE="$SCRIPT_DIR/cccp.sh"

# Function to extract function definitions
extract_functions() {
    local file="$1"
    # Extract only function definitions and their contents
    awk '
    BEGIN { in_function = 0; brace_count = 0 }
    /^[[:space:]]*[a-zA-Z_][a-zA-Z0-9_]*()/ {
        if (in_function == 0) {
            in_function = 1
            print
            next
        }
    }
    in_function == 1 {
        print
        if ($0 ~ /{/) brace_count++
        if ($0 ~ /}/) {
            brace_count--
            if (brace_count == 0) {
                in_function = 0
                print ""
            }
        }
    }
    ' "$file"
}

# Start with the main script header
cat > "$OUTPUT_FILE" << 'EOF'
#!/bin/sh

# =============================================================================
# Conventional Commits Compliance Program
# This script provides tools for managing git commits following conventional commit
# standards, version management, and changelog generation.
# =============================================================================

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
EOF

# Add configuration
printf "\n# =============================================================================\n" >> "$OUTPUT_FILE"
printf "# Configuration\n" >> "$OUTPUT_FILE"
printf "# =============================================================================\n" >> "$OUTPUT_FILE"
cat "$SCRIPT_DIR/src/config/config.sh" | grep -v "^[[:space:]]*#" | grep -v "^[[:space:]]*$" | grep -v "^[[:space:]]*set" | grep -v "^[[:space:]]*GIT_ROOT=" | grep -v "^[[:space:]]*GIT_HOOKS_DIR=" | grep -v "^[[:space:]]*\." >> "$OUTPUT_FILE"

# Add validation function
printf "\n# =============================================================================\n" >> "$OUTPUT_FILE"
printf "# Validation Functions\n" >> "$OUTPUT_FILE"
printf "# =============================================================================\n" >> "$OUTPUT_FILE"
extract_functions "$SCRIPT_DIR/src/utils/validation.sh" >> "$OUTPUT_FILE"

# Add changelog function
printf "\n# =============================================================================\n" >> "$OUTPUT_FILE"
printf "# Changelog Functions\n" >> "$OUTPUT_FILE"
printf "# =============================================================================\n" >> "$OUTPUT_FILE"
extract_functions "$SCRIPT_DIR/src/utils/changelog.sh" >> "$OUTPUT_FILE"

# Add version function
printf "\n# =============================================================================\n" >> "$OUTPUT_FILE"
printf "# Version Functions\n" >> "$OUTPUT_FILE"
printf "# =============================================================================\n" >> "$OUTPUT_FILE"
extract_functions "$SCRIPT_DIR/src/utils/version.sh" >> "$OUTPUT_FILE"

# Add hooks function
printf "\n# =============================================================================\n" >> "$OUTPUT_FILE"
printf "# Hooks Functions\n" >> "$OUTPUT_FILE"
printf "# =============================================================================\n" >> "$OUTPUT_FILE"
extract_functions "$SCRIPT_DIR/src/hooks/hooks.sh" >> "$OUTPUT_FILE"

# Add commit-msg hook function
printf "\n# =============================================================================\n" >> "$OUTPUT_FILE"
printf "# Commit Message Hook Function\n" >> "$OUTPUT_FILE"
printf "# =============================================================================\n" >> "$OUTPUT_FILE"
extract_functions "$SCRIPT_DIR/src/hooks/commit_msg.sh" >> "$OUTPUT_FILE"

# Add post-commit hook function
printf "\n# =============================================================================\n" >> "$OUTPUT_FILE"
printf "# Post Commit Hook Function\n" >> "$OUTPUT_FILE"
printf "# =============================================================================\n" >> "$OUTPUT_FILE"
extract_functions "$SCRIPT_DIR/src/hooks/post_commit.sh" >> "$OUTPUT_FILE"

# Add help function
printf "\n# =============================================================================\n" >> "$OUTPUT_FILE"
printf "# Help Functions\n" >> "$OUTPUT_FILE"
printf "# =============================================================================\n" >> "$OUTPUT_FILE"
extract_functions "$SCRIPT_DIR/src/utils/help.sh" >> "$OUTPUT_FILE"

# Add commit function
printf "\n# =============================================================================\n" >> "$OUTPUT_FILE"
printf "# Commit Functions\n" >> "$OUTPUT_FILE"
printf "# =============================================================================\n" >> "$OUTPUT_FILE"
extract_functions "$SCRIPT_DIR/src/utils/commit.sh" >> "$OUTPUT_FILE"

# Add main function
printf "\n# =============================================================================\n" >> "$OUTPUT_FILE"
printf "# Main script entry point\n" >> "$OUTPUT_FILE"
printf "# =============================================================================\n" >> "$OUTPUT_FILE"
cat >> "$OUTPUT_FILE" << 'EOF'
main() {
    if [ -n "${1:-}" ]; then
       command="$1"
    else
       command=""
    fi
    current_hook=$(basename "$0")
    
    # Check if running as a hook
    case " $GIT_HOOKS_LIST " in
        *" $current_hook "*)
            command="$current_hook"
            ;;
    esac
    
    case "$command" in
        "git")
            echo "Git command handler"
            exit 0
            ;;
        "commit")
            commit "$2"
            exit 0
            ;;
        "install")
            install_git_hooks
            ;;
        "version")
            generate_version_info
            exit 0
            ;;
        "changelog")
            generate_changelog
            exit 0
            ;;
        "commit-msg")
            commit_msg "$@"
            exit 0
            ;;
        "post-commit")
            post_commit
            exit 0
            ;;
        "update")
            update_script
            exit 0
            ;;
        *)
            echo "Usage: $0 [git|commit|install|version|changelog|commit-msg|post-commit|update]"
            exit 1
            ;;
    esac
}

# Execute main function
main "$@"
EOF

# Make the combined file executable
chmod +x "$OUTPUT_FILE"

echo "Combined script created at: $OUTPUT_FILE" 
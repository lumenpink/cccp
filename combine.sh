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
    BEGIN { in_function = 0 }
    /^[a-zA-Z_][a-zA-Z0-9_]*[[:space:]]*\(\)/ {
        in_function = 1
        print
        next
    }
    in_function == 1 {
        print
        if ($0 ~ /^}/) {
            in_function = 0
            print ""
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

# Verify required tools
for tool in git sed grep date cut tr; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "Error: Required tool '$tool' is not installed or not in PATH." >&2
        echo "Please install $tool to use cccp.sh." >&2
        exit 1
    fi
done

# Find the git root directory
GIT_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || true)"
if [ -n "$GIT_ROOT" ]; then
    GIT_HOOKS_DIR="$GIT_ROOT/.git/hooks"
else
    GIT_HOOKS_DIR=""
fi
EOF

# Add configuration
printf "\n# =============================================================================\n" >> "$OUTPUT_FILE"
printf "# Configuration\n" >> "$OUTPUT_FILE"
printf "# =============================================================================\n" >> "$OUTPUT_FILE"
cat "$SCRIPT_DIR/src/config/config.sh" | grep -v "^[[:space:]]*#" | grep -v "^[[:space:]]*$" | grep -v "^[[:space:]]*set" | grep -v "^[[:space:]]*GIT_ROOT=" | grep -v "^[[:space:]]*GIT_HOOKS_DIR=" | grep -v "config_manager.sh" | grep -v "load_hierarchical_config" >> "$OUTPUT_FILE"

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

# Add tag function
printf "\n# =============================================================================\n" >> "$OUTPUT_FILE"
printf "# Tag Functions\n" >> "$OUTPUT_FILE"
printf "# =============================================================================\n" >> "$OUTPUT_FILE"
extract_functions "$SCRIPT_DIR/src/utils/tag.sh" >> "$OUTPUT_FILE"

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

# Add config manager functions
printf "\n# =============================================================================\n" >> "$OUTPUT_FILE"
printf "# Config Manager Functions\n" >> "$OUTPUT_FILE"
printf "# =============================================================================\n" >> "$OUTPUT_FILE"
extract_functions "$SCRIPT_DIR/src/utils/config_manager.sh" >> "$OUTPUT_FILE"

# Add update function
printf "\n# =============================================================================\n" >> "$OUTPUT_FILE"
printf "# Update Functions\n" >> "$OUTPUT_FILE"
printf "# =============================================================================\n" >> "$OUTPUT_FILE"
extract_functions "$SCRIPT_DIR/src/utils/update.sh" >> "$OUTPUT_FILE"

# Add status function
printf "\n# =============================================================================\n" >> "$OUTPUT_FILE"
printf "# Status Functions\n" >> "$OUTPUT_FILE"
printf "# =============================================================================\n" >> "$OUTPUT_FILE"
extract_functions "$SCRIPT_DIR/src/utils/status.sh" >> "$OUTPUT_FILE"

# Add lint function
printf "\n# =============================================================================\n" >> "$OUTPUT_FILE"
printf "# Lint Functions\n" >> "$OUTPUT_FILE"
printf "# =============================================================================\n" >> "$OUTPUT_FILE"
extract_functions "$SCRIPT_DIR/src/utils/lint.sh" >> "$OUTPUT_FILE"

# Add soviet easter egg function
printf "\n# =============================================================================\n" >> "$OUTPUT_FILE"
printf "# Soviet Easter Egg\n" >> "$OUTPUT_FILE"
printf "# =============================================================================\n" >> "$OUTPUT_FILE"
extract_functions "$SCRIPT_DIR/src/utils/soviet.sh" >> "$OUTPUT_FILE"

# Add main function
printf "\n# =============================================================================\n" >> "$OUTPUT_FILE"
printf "# Main script entry point\n" >> "$OUTPUT_FILE"
printf "# =============================================================================\n" >> "$OUTPUT_FILE"
cat >> "$OUTPUT_FILE" << 'EOF'
main() {
    # Load configuration hierarchy (Defaults < Global < Local < Environment)
    if command -v load_hierarchical_config >/dev/null 2>&1; then
        load_hierarchical_config
    fi

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

    # Hook version audit in git repositories
    if [ -n "${GIT_ROOT:-}" ] && command -v check_hook_version >/dev/null 2>&1; then
        case "$command" in
            "commit"|"commit-msg"|"post-commit")
                check_hook_version || true
                ;;
        esac
    fi

    # Non-blocking periodic update check on interactive user commands
    if command -v check_auto_update >/dev/null 2>&1; then
        case "$command" in
            "commit"|"version"|"tag"|"changelog"|"config"|"status"|"lint")
                check_auto_update || true
                ;;
        esac
    fi
    
    case "$command" in
        "git")
            echo "Git command handler"
            exit 0
            ;;
        "config")
            shift || true
            cmd_config "$@"
            exit 0
            ;;
        "status")
            shift || true
            show_status "$@"
            exit 0
            ;;
        "lint")
            shift || true
            lint_commits "$@"
            exit 0
            ;;
        "commit")
            shift || true
            commit "$@"
            exit 0
            ;;
        "install")
            shift || true
            install_cccp "$@"
            exit 0
            ;;
        "version")
            case "${2:-}" in
                "-h"|"--help")
                    show_help "version"
                    exit 0
                    ;;
            esac
            generate_version_info
            exit 0
            ;;
        "tag")
            shift || true
            create_tag "$@"
            exit 0
            ;;
        "changelog")
            case "${2:-}" in
                "-h"|"--help")
                    show_help "changelog"
                    exit 0
                    ;;
            esac
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
            shift || true
            update_script "$@"
            exit 0
            ;;
        "soviet"|"sputnik"|"anthem"|"gosplan")
            show_soviet
            exit 0
            ;;
        "help"|"-h"|"--help")
            show_help "${2:-}"
            exit 0
            ;;
        *)
            echo "Usage: $0 [git|commit|install|config|status|lint|version|tag|changelog|commit-msg|post-commit|update|help]"
            echo "Run '$0 help' or '$0 help <command>' for more information."
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
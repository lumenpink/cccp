#!/bin/sh

# =============================================================================
# Script to combine all modular files into a single file
# =============================================================================

# Get the script's directory
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
OUTPUT_FILE="$SCRIPT_DIR/cccp.sh"

if [ ! -d "$SCRIPT_DIR/src/utils" ]; then
    echo "Notice: Pure native Rust architecture active. Modular shell utils removed."
    exit 0
fi

# Function to extract function definitions
extract_functions() {
    local file="$1"
    # Extract only function definitions and their contents
    awk '
    BEGIN { in_function = 0; in_heredoc = 0 }
    /^[a-zA-Z_][a-zA-Z0-9_]*[[:space:]]*\(\)/ && !in_function {
        in_function = 1
        print
        next
    }
    in_function == 1 {
        if (!in_heredoc && /<<[[:space:]]*[\x27"]?EOF[\x27"]?/) {
            in_heredoc = 1
        } else if (in_heredoc && /^EOF$/) {
            in_heredoc = 0
        }
        print
        if (!in_heredoc && /^}/) {
            in_function = 0
            print ""
        }
    }
    ' "$file"
}

# Synchronize and read VERSION file before compilation
if [ ! -f "$SCRIPT_DIR/VERSION" ] || [ ! -s "$SCRIPT_DIR/VERSION" ]; then
    if [ -f "$SCRIPT_DIR/src/utils/version.sh" ] && git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
        (
            cd "$SCRIPT_DIR"
            . "$SCRIPT_DIR/src/config/config.sh"
            . "$SCRIPT_DIR/src/utils/version.sh"
            generate_version_info >/dev/null 2>&1 || true
        )
    fi
fi

CCCP_VERSION="2.0.0"
if [ -f "$SCRIPT_DIR/VERSION" ]; then
    CCCP_VERSION=$(head -n 1 "$SCRIPT_DIR/VERSION" | tr -d ' \r\n')
fi

# Start with the main script header
cat > "$OUTPUT_FILE" << EOF
#!/bin/sh

# =============================================================================
# Conventional Commits Compliance Program
# This script provides tools for managing git commits following conventional commit
# standards, version management, and changelog generation.
# =============================================================================

# Enable error handling
set -eu

# Script Version (Single Source of Truth, synchronized from VERSION)
CCCP_VERSION="$CCCP_VERSION"
export CCCP_VERSION

# Verify required tools
for tool in git sed grep date cut tr; do
    if ! command -v "\$tool" >/dev/null 2>&1; then
        echo "Error: Required tool '\$tool' is not installed or not in PATH." >&2
        echo "Please install \$tool to use cccp.sh." >&2
        exit 1
    fi
done

# Find the git root directory
GIT_ROOT="\$(git rev-parse --show-toplevel 2>/dev/null || true)"
if [ -n "\$GIT_ROOT" ]; then
    GIT_HOOKS_DIR="\$GIT_ROOT/.git/hooks"
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

# Add push function
printf "\n# =============================================================================\n" >> "$OUTPUT_FILE"
printf "# Push Functions\n" >> "$OUTPUT_FILE"
printf "# =============================================================================\n" >> "$OUTPUT_FILE"
extract_functions "$SCRIPT_DIR/src/utils/push.sh" >> "$OUTPUT_FILE"

# Add repo function
printf "\n# =============================================================================\n" >> "$OUTPUT_FILE"
printf "# Repository & Forge Resolver Functions\n" >> "$OUTPUT_FILE"
printf "# =============================================================================\n" >> "$OUTPUT_FILE"
extract_functions "$SCRIPT_DIR/src/utils/repo.sh" >> "$OUTPUT_FILE"

# Add comrades function
printf "\n# =============================================================================\n" >> "$OUTPUT_FILE"
printf "# Comrades & Unified HTTP Dispatcher Functions\n" >> "$OUTPUT_FILE"
printf "# =============================================================================\n" >> "$OUTPUT_FILE"
extract_functions "$SCRIPT_DIR/src/utils/comrades.sh" >> "$OUTPUT_FILE"

# Add tickets function
printf "\n# =============================================================================\n" >> "$OUTPUT_FILE"
printf "# Tickets Functions\n" >> "$OUTPUT_FILE"
printf "# =============================================================================\n" >> "$OUTPUT_FILE"
extract_functions "$SCRIPT_DIR/src/utils/tickets.sh" >> "$OUTPUT_FILE"

# Add agents function
printf "\n# =============================================================================\n" >> "$OUTPUT_FILE"
printf "# Agents Functions\n" >> "$OUTPUT_FILE"
printf "# =============================================================================\n" >> "$OUTPUT_FILE"
extract_functions "$SCRIPT_DIR/src/utils/agents.sh" >> "$OUTPUT_FILE"

# Add codeberg function
printf "\n# =============================================================================\n" >> "$OUTPUT_FILE"
printf "# Codeberg Functions\n" >> "$OUTPUT_FILE"
printf "# =============================================================================\n" >> "$OUTPUT_FILE"
extract_functions "$SCRIPT_DIR/src/utils/codeberg.sh" >> "$OUTPUT_FILE"

# Add catalog function
printf "\n# =============================================================================\n" >> "$OUTPUT_FILE"
printf "# Catalog Functions\n" >> "$OUTPUT_FILE"
printf "# =============================================================================\n" >> "$OUTPUT_FILE"
extract_functions "$SCRIPT_DIR/src/utils/catalog.sh" >> "$OUTPUT_FILE"

# Add diva function
printf "\n# =============================================================================\n" >> "$OUTPUT_FILE"
printf "# Diva Functions\n" >> "$OUTPUT_FILE"
printf "# =============================================================================\n" >> "$OUTPUT_FILE"
extract_functions "$SCRIPT_DIR/src/utils/diva.sh" >> "$OUTPUT_FILE"

# Add matraca function
printf "\n# =============================================================================\n" >> "$OUTPUT_FILE"
printf "# Matraca Mobile Web REPL & Filesystem Queue Functions\n" >> "$OUTPUT_FILE"
printf "# =============================================================================\n" >> "$OUTPUT_FILE"
extract_functions "$SCRIPT_DIR/src/utils/matraca.sh" >> "$OUTPUT_FILE"

# Add web function
printf "\n# =============================================================================\n" >> "$OUTPUT_FILE"
printf "# Web Server & Bridge Functions\n" >> "$OUTPUT_FILE"
printf "# =============================================================================\n" >> "$OUTPUT_FILE"
extract_functions "$SCRIPT_DIR/src/utils/web.sh" >> "$OUTPUT_FILE"

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

# Add gost function
printf "\n# =============================================================================\n" >> "$OUTPUT_FILE"
printf "# GOST Compliance & Audit Functions\n" >> "$OUTPUT_FILE"
printf "# =============================================================================\n" >> "$OUTPUT_FILE"
extract_functions "$SCRIPT_DIR/src/utils/gost.sh" >> "$OUTPUT_FILE"

# Add newrepo function
printf "\n# =============================================================================\n" >> "$OUTPUT_FILE"
printf "# Repository Scaffolding (GOST Standard) Functions\n" >> "$OUTPUT_FILE"
printf "# =============================================================================\n" >> "$OUTPUT_FILE"
extract_functions "$SCRIPT_DIR/src/utils/newrepo.sh" >> "$OUTPUT_FILE"

# Add serve function
printf "\n# =============================================================================\n" >> "$OUTPUT_FILE"
printf "# Unified Server Functions (Static, Dev & Demo)\n" >> "$OUTPUT_FILE"
printf "# =============================================================================\n" >> "$OUTPUT_FILE"
extract_functions "$SCRIPT_DIR/src/utils/serve.sh" >> "$OUTPUT_FILE"

# Add act local CI runner functions
printf "\n# =============================================================================\n" >> "$OUTPUT_FILE"
printf "# Act Local CI Runner & Pre-Push Gate\n" >> "$OUTPUT_FILE"
printf "# =============================================================================\n" >> "$OUTPUT_FILE"
extract_functions "$SCRIPT_DIR/src/utils/act.sh" >> "$OUTPUT_FILE"

# Add dependencies manager functions
printf "\n# =============================================================================\n" >> "$OUTPUT_FILE"
printf "# Dependencies Manager Functions\n" >> "$OUTPUT_FILE"
printf "# =============================================================================\n" >> "$OUTPUT_FILE"
extract_functions "$SCRIPT_DIR/src/utils/deps.sh" >> "$OUTPUT_FILE"

# Add interactive commit function
printf "\n# =============================================================================\n" >> "$OUTPUT_FILE"
printf "# Interactive Commit Wizard\n" >> "$OUTPUT_FILE"
printf "# =============================================================================\n" >> "$OUTPUT_FILE"
extract_functions "$SCRIPT_DIR/src/utils/interactive.sh" >> "$OUTPUT_FILE"

# Add completion function
printf "\n# =============================================================================\n" >> "$OUTPUT_FILE"
printf "# Completion Generator\n" >> "$OUTPUT_FILE"
printf "# =============================================================================\n" >> "$OUTPUT_FILE"
extract_functions "$SCRIPT_DIR/src/utils/completion.sh" >> "$OUTPUT_FILE"

# Add soviet easter egg function
printf "\n# =============================================================================\n" >> "$OUTPUT_FILE"
printf "# Soviet Easter Egg\n" >> "$OUTPUT_FILE"
printf "# =============================================================================\n" >> "$OUTPUT_FILE"
extract_functions "$SCRIPT_DIR/src/utils/soviet.sh" >> "$OUTPUT_FILE"

# Add main function
printf "\n# =============================================================================\n" >> "$OUTPUT_FILE"
printf "# Main script entry point\n" >> "$OUTPUT_FILE"
printf "# =============================================================================\n" >> "$OUTPUT_FILE"
extract_functions "$SCRIPT_DIR/src/main.sh" >> "$OUTPUT_FILE"

# Execute main function (guarded to allow sourcing in tests)
cat >> "$OUTPUT_FILE" << 'EOF'
if [ "${CCCP_SOURCED:-0}" != "1" ]; then
    main "$@"
fi
EOF

# Make the combined file executable
chmod +x "$OUTPUT_FILE"

echo "Combined script created at: $OUTPUT_FILE" 
#!/bin/sh

# Enable error handling
set -eu

# Find the git root directory
GIT_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || true)"

# Source the configuration if available
if [ -n "$GIT_ROOT" ] && [ -f "$GIT_ROOT/src/config/config.sh" ]; then
    . "$GIT_ROOT/src/config/config.sh"
fi

# -----------------------------------------------------------------------------
# Install git hooks as portable wrapper scripts with version tracking
# -----------------------------------------------------------------------------
install_git_hooks() {
    GIT_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || true)"
    if [ -z "$GIT_ROOT" ]; then
        echo "Error: Not a git repository. This command must be executed within a valid Git repository." >&2
        return 1
    fi

    GIT_HOOKS_DIR="$GIT_ROOT/.git/hooks"
    GIT_HOOKS_LIST="${GIT_HOOKS_LIST:-commit-msg post-commit}"

    # Determine current cccp version to stamp in hook
    current_version="0.0.1"
    if [ -f "$GIT_ROOT/VERSION" ]; then
        current_version=$(head -n 1 "$GIT_ROOT/VERSION" | tr -d ' \r\n')
    elif command -v cccp >/dev/null 2>&1; then
        current_version=$(cccp version 2>/dev/null | head -n 1 || echo "0.0.1")
    fi

    # Create hooks directory if it doesn't exist
    mkdir -p "$GIT_HOOKS_DIR"

    # Process each hook in the list
    for hook in $GIT_HOOKS_LIST; do
        hook_path="$GIT_HOOKS_DIR/$hook"

        # Check if the hook is already an up-to-date cccp wrapper script
        if [ -f "$hook_path" ] && [ ! -L "$hook_path" ] && grep -q "# cccp-hook-version: $current_version" "$hook_path" 2>/dev/null; then
            echo "Hook already installed: $hook_path"
            continue
        fi

        # Handle existing hook (whether regular file or symlink)
        if [ -e "$hook_path" ] || [ -L "$hook_path" ]; then
            backup_name="$hook_path.old"
            counter=1
            while [ -e "$backup_name" ] || [ -L "$backup_name" ]; do
                backup_name="$hook_path.old.$counter"
                counter=$((counter + 1))
            done
            mv "$hook_path" "$backup_name"
            echo "Backed up existing hook to: $backup_name"
        fi

        # Write standalone wrapper script
        cat > "$hook_path" <<EOF
#!/bin/sh
# cccp-hook-version: $current_version

# Execute cccp from PATH if installed, or fallback to repo root script
if command -v cccp >/dev/null 2>&1; then
    exec cccp $hook "\$@"
elif [ -x "$GIT_ROOT/cccp.sh" ]; then
    exec "$GIT_ROOT/cccp.sh" $hook "\$@"
elif [ -x "./cccp.sh" ]; then
    exec ./cccp.sh $hook "\$@"
else
    echo "Error: cccp is not installed in PATH or repository root." >&2
    echo "Please install cccp in your PATH or place cccp.sh in repository root." >&2
    exit 1
fi
EOF
        chmod +x "$hook_path"
    done

    echo "Successfully installed git hooks!"
    echo "Hooks configured with cccp version: $current_version"
}
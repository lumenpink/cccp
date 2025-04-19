#!/bin/sh

# Enable error handling
set -eu

# Find the git root directory
GIT_ROOT="$(git rev-parse --show-toplevel 2>/dev/null)"
if [ -z "$GIT_ROOT" ]; then
    echo "Error: Not a git repository" >&2
    exit 1
fi

# Source the configuration
. "$GIT_ROOT/src/config/config.sh"

# -----------------------------------------------------------------------------
# Install git hooks
# -----------------------------------------------------------------------------
install_git_hooks() {
    # Create hooks directory if it doesn't exist
    mkdir -p "$GIT_HOOKS_DIR"

    # Configure git to use the local hooks directory
    git config --global core.hooksPath "$GIT_HOOKS_DIR"

    # Process each hook in the list
    for hook in $GIT_HOOKS_LIST; do
        # Check if the hook is already a symlink to our script
        if [ -L "$GIT_HOOKS_DIR/$hook" ] && [ "$(readlink "$GIT_HOOKS_DIR/$hook")" = "$GIT_ROOT/$GIT_HOOK_FILE" ]; then
            echo "Hook already installed: $GIT_HOOKS_DIR/$hook"
            continue
        fi
        
        # Handle existing hook (whether it's a file or a symlink)
        if [ -e "$GIT_HOOKS_DIR/$hook" ] || [ -L "$GIT_HOOKS_DIR/$hook" ]; then
            # Find an available backup name for the existing hook
            backup_name="$GIT_HOOKS_DIR/$hook.old"
            counter=1
            
            while [ -e "$backup_name" ] || [ -L "$backup_name" ]; do
                backup_name="$GIT_HOOKS_DIR/$hook.old.$counter"
                counter=$((counter + 1))
            done
            
            # Backup existing hook
            mv "$GIT_HOOKS_DIR/$hook" "$backup_name"
            echo "Backed up existing hook to: $backup_name"
        fi

        # Create symbolic link to our script        
        ln -s "$GIT_ROOT/$GIT_HOOK_FILE" "$GIT_HOOKS_DIR/$hook"
    done

    echo "Successfully installed git hooks!"
    echo "You can now use 'git cc' to commit your changes."
} 
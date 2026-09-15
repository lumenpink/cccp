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
# Install cccp executable into PATH (/usr/local/bin for root, ~/.local/bin for users)
# -----------------------------------------------------------------------------
install_global_binary() {
    # 1. Determine source file
    source_bin=""
    if [ -n "${1:-}" ] && [ -f "$1" ]; then
        source_bin="$1"
    elif [ -f "${0:-}" ] && [ -s "${0:-}" ]; then
        source_bin="${0:-}"
    elif [ -n "${GIT_ROOT:-}" ] && [ -f "$GIT_ROOT/cccp.sh" ]; then
        source_bin="$GIT_ROOT/cccp.sh"
    elif [ -f "./cccp.sh" ]; then
        source_bin="./cccp.sh"
    fi

    if [ -z "$source_bin" ] || [ ! -f "$source_bin" ]; then
        tmp_download="$(mktemp)"
        update_url="${UPDATE_URL:-https://github.com/lumenpink/cccp.sh/raw/refs/heads/main/cccp.sh}"
        if command -v curl >/dev/null 2>&1; then
            curl -fsSL "$update_url" -o "$tmp_download" 2>/dev/null || true
        elif command -v wget >/dev/null 2>&1; then
            wget -q "$update_url" -O "$tmp_download" 2>/dev/null || true
        fi

        if [ -s "$tmp_download" ]; then
            source_bin="$tmp_download"
        else
            echo "Error: Could not locate or download cccp.sh to install." >&2
            return 1
        fi
    fi

    # 2. Determine target destination based on EUID / uid
    user_id=$(id -u 2>/dev/null || echo "1000")
    if [ "$user_id" -eq 0 ]; then
        target_dir="/usr/local/bin"
    else
        target_dir="${XDG_BIN_HOME:-$HOME/.local/bin}"
    fi

    mkdir -p "$target_dir"
    target_bin="$target_dir/cccp"

    # 3. Copy binary and make executable
    cp "$source_bin" "$target_bin"
    chmod +x "$target_bin"
    echo "Successfully installed cccp to: $target_bin"

    # 4. Check if target_dir is in PATH
    case ":$PATH:" in
        *":$target_dir:"*)
            in_path=1
            ;;
        *)
            in_path=0
            ;;
    esac

    if [ $in_path -eq 0 ]; then
        echo ""
        echo "Notice: '$target_dir' is not currently in your \$PATH."

        # Check ~/.bashrc
        bashrc="$HOME/.bashrc"
        if [ -f "$bashrc" ] || [ ! -f "$HOME/.zshrc" ]; then
            if [ ! -f "$bashrc" ] || ! grep -q "$target_dir" "$bashrc" 2>/dev/null; then
                mkdir -p "$(dirname "$bashrc")"
                printf "\n# Added by cccp installer\nexport PATH=\"%s:\$PATH\"\n" "$target_dir" >> "$bashrc"
                echo "Added '$target_dir' to $bashrc"
            fi
        fi

        # Check ~/.zshrc
        zshrc="$HOME/.zshrc"
        if [ -f "$zshrc" ]; then
            if ! grep -q "$target_dir" "$zshrc" 2>/dev/null; then
                printf "\n# Added by cccp installer\nexport PATH=\"%s:\$PATH\"\n" "$target_dir" >> "$zshrc"
                echo "Added '$target_dir' to $zshrc"
            fi
        fi

        echo ""
        echo "To use 'cccp' immediately in this terminal session, run:"
        echo "    export PATH=\"$target_dir:\$PATH\""
        echo "Or open a new terminal window."
    else
        echo "You can now run 'cccp' from anywhere!"
    fi

    return 0
}

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

# -----------------------------------------------------------------------------
# Dispatcher for install command
# -----------------------------------------------------------------------------
install_cccp() {
    is_global=0

    while [ $# -gt 0 ]; do
        case "$1" in
            --global|-g)
                is_global=1
                shift
                ;;
            -h|--help)
                if command -v show_help >/dev/null 2>&1; then
                    show_help "install"
                else
                    echo "Usage: cccp install [--global|-g]"
                fi
                return 0
                ;;
            *)
                echo "Error: Unexpected argument '$1'" >&2
                return 1
                ;;
        esac
    done

    if [ $is_global -eq 1 ]; then
        install_global_binary
    else
        install_git_hooks
    fi
}
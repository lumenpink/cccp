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
# Canonical hook wrapper template generator
# -----------------------------------------------------------------------------
get_canonical_hook_content() {
    hook_name="$1"
    ver="${2:-${CCCP_VERSION:-2.0.0}}"
    root_dir="${3:-${GIT_ROOT:-}}"
    cat <<EOF
#!/bin/sh
# cccp-hook-version: $ver

# Execute cccp from PATH if installed, or fallback to repo root script
if command -v cccp >/dev/null 2>&1; then
    exec cccp $hook_name "\$@"
elif [ -x "$root_dir/cccp.sh" ]; then
    exec "$root_dir/cccp.sh" $hook_name "\$@"
elif [ -x "./cccp.sh" ]; then
    exec ./cccp.sh $hook_name "\$@"
else
    echo "Error: cccp is not installed in PATH or repository root." >&2
    echo "Please install cccp in your PATH or place cccp.sh in repository root." >&2
    exit 1
fi
EOF
}

# -----------------------------------------------------------------------------
# Detect framework or runtime of non-CCCP hook
# -----------------------------------------------------------------------------
detect_hook_framework() {
    file="$1"
    if [ ! -f "$file" ]; then
        echo "none"
        return 0
    fi
    if grep -qi "husky" "$file" 2>/dev/null; then
        echo "Husky"
    elif grep -qi "lefthook" "$file" 2>/dev/null; then
        echo "Lefthook"
    elif grep -qi "pre-commit" "$file" 2>/dev/null; then
        echo "pre-commit (Python framework)"
    elif grep -qi "overcommit" "$file" 2>/dev/null; then
        echo "Overcommit"
    else
        first_line=$(head -n 1 "$file" 2>/dev/null || true)
        case "$first_line" in
            *sh*) echo "Custom Shell Script" ;;
            *node*|*js*) echo "Custom Node.js Script" ;;
            *python*) echo "Custom Python Script" ;;
            *) echo "Custom Executable / Script" ;;
        esac
    fi
}

# -----------------------------------------------------------------------------
# Discover hook backups in hooks directory
# -----------------------------------------------------------------------------
get_hook_backups() {
    hook_path="$1"
    dir=$(dirname "$hook_path")
    base=$(basename "$hook_path")
    backups=""
    for b in "$hook_path.old" "$hook_path.old."*; do
        if [ -f "$b" ] || [ -L "$b" ]; then
            backups="${backups}$(basename "$b") "
        fi
    done
    echo "$backups" | sed 's/[[:space:]]*$//'
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
    GIT_HOOKS_LIST="${GIT_HOOKS_LIST:-commit-msg post-commit pre-push reference-transaction}"

    # Determine current cccp version to stamp in hook
    current_version="${CCCP_VERSION:-2.0.0}"

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
            while { [ -e "$backup_name" ] || [ -L "$backup_name" ]; } && [ "$counter" -le 100 ]; do
                backup_name="$hook_path.old.$counter"
                counter=$((counter + 1))
            done
            mv "$hook_path" "$backup_name"
            echo "Backed up existing hook to: $backup_name"
        fi

        # Write standalone wrapper script
        get_canonical_hook_content "$hook" "$current_version" > "$hook_path"
        chmod +x "$hook_path"
    done

    echo "Successfully installed git hooks!"
    echo "Hooks configured with cccp version: $current_version"
}

# -----------------------------------------------------------------------------
# Deep audit of repository Git hooks
# -----------------------------------------------------------------------------
audit_git_hooks() {
    GIT_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || true)"
    if [ -z "$GIT_ROOT" ]; then
        echo "Error: Not a git repository. This command must be executed within a valid Git repository." >&2
        return 1
    fi

    hooks_dir="${GIT_HOOKS_DIR:-$GIT_ROOT/.git/hooks}"
    curr_ver="${CCCP_VERSION:-2.0.0}"
    hooks_list="${GIT_HOOKS_LIST:-commit-msg post-commit pre-push reference-transaction}"

    echo "========================================================"
    echo " ★ CCCP Git Hooks Inspectorate (Komissariat Audit) ★"
    echo "========================================================"
    echo " Repository    : $GIT_ROOT"
    echo " Hooks Dir     : $hooks_dir"
    echo " CCCP Version  : $curr_ver"
    echo ""

    for hook in $hooks_list; do
        hook_path="$hooks_dir/$hook"
        echo " Hook: $hook"
        if [ ! -e "$hook_path" ] && [ ! -L "$hook_path" ]; then
            echo "   Status      : Missing / Not installed"
            echo "   Intervention: Run 'cccp install' to install standard CCCP hook."
            echo ""
            continue
        fi

        # Check if symlink
        if [ -L "$hook_path" ]; then
            link_target=$(ls -l "$hook_path" 2>/dev/null | sed 's/.*-> //')
            echo "   Type        : Symbolic link (-> $link_target)"
        fi

        # Check if CCCP wrapper
        if grep -q "# cccp-hook-version:" "$hook_path" 2>/dev/null; then
            hook_ver=$(sed -n 's/^# cccp-hook-version:[[:space:]]*//p' "$hook_path" | head -n 1)
            if [ "$hook_ver" = "$curr_ver" ]; then
                echo "   Status      : Synchronized (v$hook_ver)"
                echo "   Details     : Up to date with active CCCP version."
            else
                echo "   Status      : Outdated CCCP Wrapper (v$hook_ver vs current v$curr_ver)"
                echo "   Intervention: Run 'cccp install' to upgrade wrapper to v$curr_ver."
                echo "                 Run 'cccp hooks diff $hook' to view changes."
            fi
        else
            framework=$(detect_hook_framework "$hook_path")
            echo "   Status      : Custom / Non-CCCP"
            echo "   Framework   : $framework"
            if grep -q "cccp" "$hook_path" 2>/dev/null; then
                echo "   Chains CCCP : Yes (invokes cccp)"
            else
                echo "   Chains CCCP : No"
            fi
            backups=$(get_hook_backups "$hook_path")
            if [ -n "$backups" ]; then
                echo "   Backups     : $backups"
            fi
            echo "   Intervention:"
            echo "     - To replace with CCCP: Run 'cccp install' (current hook will be backed up)."
            echo "     - To chain CCCP inside this hook: Add 'cccp $hook \"\$@\"' to $hook_path."
            echo "     - To inspect differences: Run 'cccp hooks diff $hook'."
            if [ -n "$backups" ]; then
                first_backup=$(echo "$backups" | cut -d' ' -f1)
                echo "     - To restore prior backup: Run 'mv $hooks_dir/$first_backup $hook_path'."
            fi
        fi
        echo ""
    done
    echo "========================================================"
    return 0
}

# -----------------------------------------------------------------------------
# Diff installed git hooks against canonical CCCP wrappers
# -----------------------------------------------------------------------------
diff_git_hooks() {
    GIT_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || true)"
    if [ -z "$GIT_ROOT" ]; then
        echo "Error: Not a git repository. This command must be executed within a valid Git repository." >&2
        return 1
    fi

    hooks_dir="${GIT_HOOKS_DIR:-$GIT_ROOT/.git/hooks}"
    curr_ver="${CCCP_VERSION:-2.0.0}"
    target_hook="${1:-}"

    if [ -n "$target_hook" ]; then
        hooks_to_diff="$target_hook"
    else
        hooks_to_diff="${GIT_HOOKS_LIST:-commit-msg post-commit pre-push reference-transaction}"
    fi

    diff_cmd="diff -u"
    if ! command -v diff >/dev/null 2>&1; then
        echo "Error: 'diff' tool is not available in system PATH." >&2
        return 1
    fi

    for hook in $hooks_to_diff; do
        hook_path="$hooks_dir/$hook"
        echo "=== Diff: $hook ($hook_path vs canonical CCCP v$curr_ver) ==="
        if [ ! -e "$hook_path" ] && [ ! -L "$hook_path" ]; then
            echo "Hook file '$hook_path' does not exist."
            echo ""
            continue
        fi

        tmp_expected="$(mktemp)"
        get_canonical_hook_content "$hook" "$curr_ver" > "$tmp_expected"

        if diff -q "$hook_path" "$tmp_expected" >/dev/null 2>&1; then
            echo "No differences found. Installed hook is identical to canonical CCCP wrapper."
        else
            $diff_cmd "$hook_path" "$tmp_expected" || true
            echo ""
            echo "Intervention Guidance:"
            if grep -q "# cccp-hook-version:" "$hook_path" 2>/dev/null; then
                echo "  - Installed hook is an older CCCP wrapper. Run 'cccp install' to upgrade."
            else
                echo "  - Installed hook is custom/non-CCCP. Run 'cccp install' to replace (with backup),"
                echo "    or manually chain 'cccp $hook \"\$@\"' within your script."
            fi
        fi
        rm -f "$tmp_expected"
        echo ""
    done

    return 0
}

# -----------------------------------------------------------------------------
# Dispatcher for hooks command
# -----------------------------------------------------------------------------
cmd_hooks() {
    if [ $# -gt 0 ]; then
        case "$1" in
            audit|check|status)
                shift
                audit_git_hooks "$@"
                return $?
                ;;
            diff)
                shift
                diff_git_hooks "$@"
                return $?
                ;;
            -h|--help)
                if command -v show_help >/dev/null 2>&1; then
                    show_help "hooks"
                else
                    echo "Usage: cccp hooks [audit|diff [hook]]"
                fi
                return 0
                ;;
            *)
                for h in ${GIT_HOOKS_LIST:-commit-msg post-commit pre-push reference-transaction}; do
                    if [ "$1" = "$h" ]; then
                        diff_git_hooks "$@"
                        return $?
                    fi
                done
                echo "Error: Unknown hooks action '$1'. Choose 'audit' or 'diff'." >&2
                return 1
                ;;
        esac
    fi
    audit_git_hooks
}

# -----------------------------------------------------------------------------
# Hook handler: pre-push
# Intercepts 'git push' to verify that any tags being pushed have a synchronized
# clean VERSION file inside their target commit.
# -----------------------------------------------------------------------------
cmd_pre_push() {
    # If stdin is a tty, nothing to validate
    [ -t 0 ] && return 0

    has_error=0
    while read -r local_ref local_sha remote_ref remote_sha || [ -n "$local_ref" ]; do
        # Ignore ref deletions
        if [ "$local_sha" = "0000000000000000000000000000000000000000" ] || [ "$local_ref" = "(delete)" ]; then
            continue
        fi

        case "$remote_ref" in
            refs/tags/*)
                tag_name="${remote_ref#refs/tags/}"
                clean_tag="${tag_name#v}"

                # Check if VERSION exists in the commit pointed to by local_sha
                ver_in_commit=$(git show "${local_sha}:VERSION" 2>/dev/null || true)
                ver_in_commit=$(echo "$ver_in_commit" | tr -d ' \r\n')

                if [ -z "$ver_in_commit" ]; then
                    echo "========================================================" >&2
                    echo " ★ CCCP Gosplan Push Quality Control (Intervention) ★" >&2
                    echo "========================================================" >&2
                    echo " Error: Tag '$tag_name' points to a commit without a VERSION file." >&2
                    echo " Target commit: $local_sha" >&2
                    echo "" >&2
                    echo " Gosplan Directive: Releases must contain an official VERSION file." >&2
                    echo " Run 'cccp tag' to generate the release commit and changelog." >&2
                    echo "========================================================" >&2
                    has_error=1
                elif echo "$ver_in_commit" | grep -qE -- "-dev|\+"; then
                    echo "========================================================" >&2
                    echo " ★ CCCP Gosplan Push Quality Control (Intervention) ★" >&2
                    echo "========================================================" >&2
                    echo " Error: Tag '$tag_name' points to a development version: '$ver_in_commit'." >&2
                    echo " Target commit: $local_sha" >&2
                    echo "" >&2
                    echo " Gosplan Directive: Pre-release/development versions cannot be pushed as release tags." >&2
                    echo " Run 'cccp tag' to generate clean release commits and synchronize CHANGELOG." >&2
                    echo "========================================================" >&2
                    has_error=1
                elif [ "$ver_in_commit" != "$clean_tag" ] && [ "$ver_in_commit" != "$tag_name" ]; then
                    echo "========================================================" >&2
                    echo " ★ CCCP Gosplan Push Quality Control (Intervention) ★" >&2
                    echo "========================================================" >&2
                    echo " Error: Tag '$tag_name' does not match VERSION file in target commit." >&2
                    echo " Expected: '$clean_tag', found in commit: '$ver_in_commit'" >&2
                    echo " Target commit: $local_sha" >&2
                    echo "" >&2
                    echo " Gosplan Directive: Tag name and VERSION file must be strictly synchronized." >&2
                    echo " Run 'cccp tag' to generate clean release commits and synchronize CHANGELOG." >&2
                    echo "========================================================" >&2
                    has_error=1
                fi
                ;;
        esac
    done

    if [ "$has_error" -eq 1 ]; then
        return 1
    fi
    return 0
}

# -----------------------------------------------------------------------------
# Hook handler: reference-transaction
# Intercepts 'git tag' at creation time to prevent tagging commits with unsynchronized
# or development VERSION files.
# -----------------------------------------------------------------------------
cmd_reference_transaction() {
    state="${1:-}"

    # Only enforce validation during 'prepared' state
    if [ "$state" != "prepared" ]; then
        return 0
    fi

    # If cccp tag is orchestrating the release, bypass check
    if [ "${CCCP_TAG_ACTIVE:-0}" -eq 1 ]; then
        return 0
    fi

    # If stdin is a tty, nothing to validate
    [ -t 0 ] && return 0

    while read -r old_sha new_sha ref_name || [ -n "$old_sha" ]; do
        # Ignore tag deletion
        if [ "$new_sha" = "0000000000000000000000000000000000000000" ]; then
            continue
        fi

        case "$ref_name" in
            refs/tags/*)
                tag_name="${ref_name#refs/tags/}"
                clean_tag="${tag_name#v}"

                # Inspect VERSION file in the commit being tagged
                ver_in_commit=$(git show "${new_sha}:VERSION" 2>/dev/null || true)
                ver_in_commit=$(echo "$ver_in_commit" | tr -d ' \r\n')

                if [ -z "$ver_in_commit" ]; then
                    echo "========================================================" >&2
                    echo " ★ CCCP Gosplan Tag Quality Control (Intervention) ★" >&2
                    echo "========================================================" >&2
                    echo " Error: Cannot create tag '$tag_name'." >&2
                    echo " The target commit ($new_sha) does not have a VERSION file." >&2
                    echo "" >&2
                    echo " Gosplan Directive: Direct git tag blocked." >&2
                    echo " Use 'cccp tag' to generate the release commit and synchronize VERSION and CHANGELOG." >&2
                    echo "========================================================" >&2
                    return 1
                elif echo "$ver_in_commit" | grep -qE -- "-dev|\+"; then
                    echo "========================================================" >&2
                    echo " ★ CCCP Gosplan Tag Quality Control (Intervention) ★" >&2
                    echo "========================================================" >&2
                    echo " Error: Cannot create tag '$tag_name'." >&2
                    echo " The target commit has a development VERSION: '$ver_in_commit'." >&2
                    echo "" >&2
                    echo " Gosplan Directive: Direct git tag blocked." >&2
                    echo " Use 'cccp tag' to create official release commits with clean SemVer." >&2
                    echo "========================================================" >&2
                    return 1
                elif [ "$ver_in_commit" != "$clean_tag" ] && [ "$ver_in_commit" != "$tag_name" ]; then
                    echo "========================================================" >&2
                    echo " ★ CCCP Gosplan Tag Quality Control (Intervention) ★" >&2
                    echo "========================================================" >&2
                    echo " Error: Cannot create tag '$tag_name'." >&2
                    echo " Tag version does not match target commit VERSION: '$ver_in_commit'." >&2
                    echo " Expected: '$clean_tag'" >&2
                    echo "" >&2
                    echo " Gosplan Directive: Direct git tag blocked." >&2
                    echo " Use 'cccp tag' to synchronize VERSION, tag, and CHANGELOG." >&2
                    echo "========================================================" >&2
                    return 1
                fi
                ;;
        esac
    done

    return 0
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
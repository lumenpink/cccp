#!/bin/sh

# Enable error handling
set -eu

# -----------------------------------------------------------------------------
# Shell Autocompletion Generator (Teleprinter Telegraph)
# -----------------------------------------------------------------------------
generate_bash_completion() {
    cat << 'EOF'
# CCCP Bash Completion
_cccp_completions() {
    local cur prev
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"

    local commands="status cz commit lint config version tag push tickets sync-tickets changelog update install completion hooks soviet help"
    local config_keys="types scopes subscopes strict_types strict_scopes strict_subscopes disable_subscopes disable_multiple_scopes default_base_version no_v update_channel update_interval_days check_updates post_push_hook tickets_file"

    if [ "$COMP_CWORD" -eq 1 ]; then
        COMPREPLY=($(compgen -W "$commands" -- "$cur"))
        return 0
    fi

    local cmd="${COMP_WORDS[1]}"
    case "$cmd" in
        tickets|sync-tickets)
            if [ "${cur#*-}" != "$cur" ]; then
                COMPREPLY=($(compgen -W "-f --file -o --owner -r --repo -m --max-bytes -t --token -h --help" -- "$cur"))
            fi
            ;;
        config)
            case "$prev" in
                --unset)
                    COMPREPLY=($(compgen -W "$config_keys" -- "$cur"))
                    return 0
                    ;;
                update_channel)
                    COMPREPLY=($(compgen -W "stable nightly" -- "$cur"))
                    return 0
                    ;;
                strict_types|strict_scopes|strict_subscopes|disable_subscopes|disable_multiple_scopes|no_v|check_updates)
                    COMPREPLY=($(compgen -W "0 1" -- "$cur"))
                    return 0
                    ;;
            esac
            if [ "${cur#*-}" != "$cur" ]; then
                COMPREPLY=($(compgen -W "--global -g --local --list -l --unset -h --help" -- "$cur"))
            else
                COMPREPLY=($(compgen -W "$config_keys" -- "$cur"))
            fi
            ;;
        commit)
            if [ "${cur#*-}" != "$cur" ]; then
                COMPREPLY=($(compgen -W "-i --interactive --strict-scopes --allow-any-scope --strict-subscopes --allow-any-subscope --disable-subscopes --enable-subscopes --disable-multiple-scopes --enable-multiple-scopes -h --help" -- "$cur"))
            fi
            ;;
        tag)
            if [ "${cur#*-}" != "$cur" ]; then
                COMPREPLY=($(compgen -W "-t --title -m --no-v -h --help" -- "$cur"))
            fi
            ;;
        update)
            case "$prev" in
                --channel)
                    COMPREPLY=($(compgen -W "stable nightly" -- "$cur"))
                    return 0
                    ;;
            esac
            if [ "${cur#*-}" != "$cur" ]; then
                COMPREPLY=($(compgen -W "--channel -h --help" -- "$cur"))
            fi
            ;;
        install)
            if [ "${cur#*-}" != "$cur" ]; then
                COMPREPLY=($(compgen -W "-g --global --profile -h --help" -- "$cur"))
            fi
            ;;
        completion)
            COMPREPLY=($(compgen -W "bash zsh fish" -- "$cur"))
            ;;
        help)
            COMPREPLY=($(compgen -W "$commands" -- "$cur"))
            ;;
    esac
}
complete -F _cccp_completions cccp cccp.sh
EOF
}

generate_zsh_completion() {
    cat << 'EOF'
#compdef cccp cccp.sh

_cccp() {
    local -a commands
    commands=(
        'status:Inspect repository health and Gosplan diagnostics'
        'cz:Interactive terminal wizard for creating commits'
        'commit:Validate formatting and create a commit'
        'lint:Lint commit messages across a git revision range'
        'config:Manage hierarchical configuration'
        'version:Generate predictive SemVer metadata'
        'tag:Create annotated tag, commit VERSION and CHANGELOG'
        'push:Execute git push and trigger optional post-push hook'
        'tickets:Sync Codeberg issues into markdown'
        'sync-tickets:Sync Codeberg issues into markdown'
        'changelog:Generate or update CHANGELOG.md'
        'update:Update cccp executable from GitHub releases'
        'install:Install to PATH or configure Git hooks'
        'completion:Generate shell autocompletion script'
        'hooks:Inspect git hooks provenance and diff wrappers'
        'soviet:Display Soviet State anthem and art'
        'help:Display documentation for any command'
    )

    local -a config_keys
    config_keys=(
        'types:Allowed commit types list'
        'scopes:Allowed commit scopes list'
        'subscopes:Allowed commit subscopes list'
        'strict_types:Enforce type matching known types list'
        'strict_scopes:Require scope to match known list'
        'strict_subscopes:Require subscope to match known list'
        'disable_subscopes:Disallow slash-delimited subscopes'
        'disable_multiple_scopes:Disallow comma-separated scopes'
        'default_base_version:Fallback SemVer when no tags exist'
        'no_v:Tag without v prefix'
        'update_channel:Update channel (stable or nightly)'
        'update_interval_days:Days between update checks'
        'check_updates:Enable automated update notifications'
        'post_push_hook:Custom script or command executed following git push'
        'tickets_file:Destination file for Codeberg issues sync'
    )

    _arguments -C \
        '1: :->command' \
        '*:: :->args'

    case $state in
        command)
            _describe -t commands 'cccp command' commands
            ;;
        args)
            case $line[1] in
                config)
                    _arguments \
                        '(-g --global)'{-g,--global}'[Target global config file]' \
                        '--local[Target local repository config file]' \
                        '(-l --list)'{-l,--list}'[List configuration key/value pairs]' \
                        '--unset[Remove a configuration key]:key:->config_keys' \
                        '1:key:->config_keys' \
                        '2:value:'
                    if [[ $state == config_keys ]]; then
                        _describe -t config_keys 'configuration keys' config_keys
                    fi
                    ;;
                commit)
                    _arguments \
                        '(-i --interactive)'{-i,--interactive}'[Launch interactive terminal wizard]' \
                        '--strict-scopes[Enforce known scopes]' \
                        '--allow-any-scope[Allow arbitrary scopes]' \
                        '--strict-subscopes[Enforce known subscopes]' \
                        '--allow-any-subscope[Allow arbitrary subscopes]' \
                        '--disable-subscopes[Disallow subscopes]' \
                        '--enable-subscopes[Enable subscopes]' \
                        '--disable-multiple-scopes[Disallow multiple scopes]' \
                        '--enable-multiple-scopes[Enable multiple scopes]' \
                        '1:commit message:'
                    ;;
                tag)
                    _arguments \
                        '(-t --title)'{-t,--title}'[Release title/description]:title:' \
                        '-m[Custom tag message]:message:' \
                        '--no-v[Tag without v prefix]' \
                        '1:version:'
                    ;;
                tickets|sync-tickets)
                    _arguments \
                        '(-f --file)'{-f,--file}'[Output markdown file]:file:_files' \
                        '(-o --owner)'{-o,--owner}'[Codeberg repository owner]:owner:' \
                        '(-r --repo)'{-r,--repo}'[Codeberg repository name]:repo:' \
                        '(-m --max-bytes)'{-m,--max-bytes}'[Maximum budget in bytes]:bytes:' \
                        '(-t --token)'{-t,--token}'[Codeberg personal access token]:token:'
                    ;;
                update)
                    _arguments \
                        '--channel[Release channel]:channel:(stable nightly)'
                    ;;
                install)
                    _arguments \
                        '(-g --global)'{-g,--global}'[Install globally to PATH]' \
                        '--profile[Update shell profile]'
                    ;;
                completion)
                    _arguments \
                        '1:shell:(bash zsh fish)'
                    ;;
                help)
                    _describe -t commands 'cccp command' commands
                    ;;
            esac
            ;;
    esac
}

_cccp "$@"
EOF
}

generate_fish_completion() {
    cat << 'EOF'
# CCCP Fish Completion

# Disable file completion by default
complete -c cccp -f
complete -c cccp.sh -f

# Main commands
complete -c cccp -n "__fish_use_subcommand" -a status -d "Inspect repository health and Gosplan diagnostics"
complete -c cccp -n "__fish_use_subcommand" -a cz -d "Interactive terminal wizard for creating commits"
complete -c cccp -n "__fish_use_subcommand" -a commit -d "Validate formatting and create a commit"
complete -c cccp -n "__fish_use_subcommand" -a lint -d "Lint commit messages across a git revision range"
complete -c cccp -n "__fish_use_subcommand" -a config -d "Manage hierarchical configuration"
complete -c cccp -n "__fish_use_subcommand" -a version -d "Generate predictive SemVer metadata"
complete -c cccp -n "__fish_use_subcommand" -a tag -d "Create annotated tag, commit VERSION and CHANGELOG"
complete -c cccp -n "__fish_use_subcommand" -a push -d "Execute git push and trigger optional post-push hook"
complete -c cccp -n "__fish_use_subcommand" -a tickets -d "Sync Codeberg issues into markdown"
complete -c cccp -n "__fish_use_subcommand" -a sync-tickets -d "Sync Codeberg issues into markdown"
complete -c cccp -n "__fish_use_subcommand" -a changelog -d "Generate or update CHANGELOG.md"
complete -c cccp -n "__fish_use_subcommand" -a update -d "Update cccp executable from GitHub releases"
complete -c cccp -n "__fish_use_subcommand" -a install -d "Install to PATH or configure Git hooks"
complete -c cccp -n "__fish_use_subcommand" -a completion -d "Generate shell autocompletion script"
complete -c cccp -n "__fish_use_subcommand" -a soviet -d "Display Soviet State anthem and art"
complete -c cccp -n "__fish_use_subcommand" -a help -d "Display documentation for any command"

# Options for commit
complete -c cccp -n "__fish_seen_subcommand_from commit" -s i -l interactive -d "Launch interactive terminal wizard"

# Options for tag
complete -c cccp -n "__fish_seen_subcommand_from tag" -s t -l title -d "Release title"
complete -c cccp -n "__fish_seen_subcommand_from tag" -l no-v -d "Tag without v prefix"

# Options for tickets
complete -c cccp -n "__fish_seen_subcommand_from tickets sync-tickets" -s f -l file -d "Output markdown file"
complete -c cccp -n "__fish_seen_subcommand_from tickets sync-tickets" -s o -l owner -d "Codeberg repository owner"
complete -c cccp -n "__fish_seen_subcommand_from tickets sync-tickets" -s r -l repo -d "Codeberg repository name"
complete -c cccp -n "__fish_seen_subcommand_from tickets sync-tickets" -s m -l max-bytes -d "Maximum budget in bytes"
complete -c cccp -n "__fish_seen_subcommand_from tickets sync-tickets" -s t -l token -d "Codeberg personal access token"

# Options for config
complete -c cccp -n "__fish_seen_subcommand_from config" -s g -l global -d "Target global config file"
complete -c cccp -n "__fish_seen_subcommand_from config" -l local -d "Target local repository config file"
complete -c cccp -n "__fish_seen_subcommand_from config" -s l -l list -d "List configuration key/value pairs"
complete -c cccp -n "__fish_seen_subcommand_from config" -l unset -d "Remove a configuration key"

# Options for completion
complete -c cccp -n "__fish_seen_subcommand_from completion" -a "bash zsh fish"
EOF
}

cmd_completion() {
    target_shell="${1:-}"

    if [ -z "$target_shell" ]; then
        if [ -n "${SHELL:-}" ]; then
            target_shell=$(basename "$SHELL")
        else
            target_shell="bash"
        fi
    fi

    case "$target_shell" in
        bash)
            generate_bash_completion
            ;;
        zsh)
            generate_zsh_completion
            ;;
        fish)
            generate_fish_completion
            ;;
        *)
            echo "Error: Unsupported shell '$target_shell'. Supported shells: bash, zsh, fish" >&2
            return 1
            ;;
    esac
    return 0
}

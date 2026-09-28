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

    local commands="status cz commit lint config version tag push tickets sync-tickets ticket issue agent update-agent sync-agent agents update-agents sync-agents codeberg set-av catalog sync-catalog diva divan web comrades camaradas comrade check gost serve repo repos act changelog update install completion hooks soviet help"
    local config_keys="types scopes subscopes strict_types strict_scopes strict_subscopes disable_subscopes disable_multiple_scopes default_base_version no_v update_channel update_interval_days check_updates post_push_hook post_push_sync post_push_sync_tickets post_push_sync_agent post_push_sync_agents tickets_file agent_md_url agents_md_url agent_md_file agents_md_file"

    if [ "$COMP_CWORD" -eq 1 ]; then
        COMPREPLY=($(compgen -W "$commands" -- "$cur"))
        return 0
    fi

    local cmd="${COMP_WORDS[1]}"
    case "$cmd" in
        diva|divan)
            if [ "${cur#*-}" != "$cur" ]; then
                COMPREPLY=($(compgen -W "--web -w --dry-run --model -m --temp -t --repo -r -h --help" -- "$cur"))
            else
                COMPREPLY=($(compgen -W "config" -- "$cur"))
            fi
            ;;
        web)
            if [ "${cur#*-}" != "$cur" ]; then
                COMPREPLY=($(compgen -W "-p --port --host -o --open -t --token --key --api-key --no-open -q --quiet -h --help" -- "$cur"))
            else
                COMPREPLY=($(compgen -W "diva status tickets" -- "$cur"))
            fi
            ;;
        check)
            if [ "${cur#*-}" != "$cur" ]; then
                COMPREPLY=($(compgen -W "--strict -C --dir -h --help" -- "$cur"))
            fi
            ;;
        gost)
            if [ "${cur#*-}" != "$cur" ]; then
                COMPREPLY=($(compgen -W "-C --dir -h --help" -- "$cur"))
            else
                COMPREPLY=($(compgen -W "sync check audit upgrade init info status help" -- "$cur"))
            fi
            ;;
        serve)
            if [ "${cur#*-}" != "$cur" ]; then
                COMPREPLY=($(compgen -W "--dev --demo -l --localhost --host -p --port -o --open -q --quiet -h --help" -- "$cur"))
            fi
            ;;
        repo|repos)
            if [ "${cur#*-}" != "$cur" ]; then
                COMPREPLY=($(compgen -W "--dirty --unpushed --issues --tickets --json -d --dir -c --catalog -f --fetch -h --help" -- "$cur"))
            else
                COMPREPLY=($(compgen -W "status list overview help" -- "$cur"))
            fi
            ;;
        act)
            if [ "${cur#*-}" != "$cur" ]; then
                COMPREPLY=($(compgen -W "-f --follow -j --job -h --help" -- "$cur"))
            else
                COMPREPLY=($(compgen -W "log logs run status help" -- "$cur"))
            fi
            ;;
        agent|update-agent|sync-agent|agents|update-agents|sync-agents)
            case "$prev" in
                -u|--url|-f|--file)
                    return 0
                    ;;
            esac
            if [ "${cur#*-}" != "$cur" ]; then
                COMPREPLY=($(compgen -W "-u --url -f --file -h --help" -- "$cur"))
            else
                COMPREPLY=($(compgen -W "update check fill set get unset" -- "$cur"))
            fi
            ;;
        codeberg)
            COMPREPLY=($(compgen -W "setup" -- "$cur"))
            if [ "$prev" = "setup" ] || [ "$cur" = "-"* ]; then
                COMPREPLY=($(compgen -W "-u --url -h --help" -- "$cur"))
            fi
            ;;
        set-av)
            return 0
            ;;
        tickets|sync-tickets)
            if [ "${cur#*-}" != "$cur" ]; then
                COMPREPLY=($(compgen -W "-f --file -o --owner -r --repo -p --provider -m --max-bytes -t --token -h --help" -- "$cur"))
            fi
            ;;
        ticket|issue|diva|divan)
            case "$prev" in
                -m|--model|-t|--temp|-r|--repo)
                    return 0
                    ;;
            esac
            if [ "${cur#*-}" != "$cur" ]; then
                COMPREPLY=($(compgen -W "--dry-run --model -m --temp -t --repo -r -h --help" -- "$cur"))
            else
                COMPREPLY=($(compgen -W "config" -- "$cur"))
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
        'tickets:Sync forge issues into markdown'
        'sync-tickets:Sync forge issues into markdown'
        'ticket:Interactive issue creation via Diva with GOST branch suggestion'
        'issue:Interactive issue creation via Diva with GOST branch suggestion'
        'diva:Gemini-powered issue triage bot with multi-forge dispatch'
        'divan:Gemini-powered issue triage bot with multi-forge dispatch'
        'catalog:Inventory workspaces, update description and sync forges'
        'comrades:Manage micro ecosystem tools (kvant, svyaz, mayak)'
        'agent:Generate and synchronize AGENTS.md directives'
        'update-agent:Generate and synchronize AGENTS.md directives'
        'sync-agent:Generate and synchronize AGENTS.md directives'
        'agents:Generate and synchronize AGENTS.md directives'
        'update-agents:Generate and synchronize AGENTS.md directives'
        'sync-agents:Generate and synchronize AGENTS.md directives'
        'set-av:Fast shortcut to configure local agent variable'
        'codeberg:Configure Codeberg integration and AGENTS template wizard'
        'catalog:Inventory workspaces, update .git/description and sync forges'
        'diva:Gemini-powered issue triage bot with multi-forge dispatch'
        'divan:Gemini-powered issue triage bot with multi-forge dispatch'
        'web:Interactive Gosplan Web Cockpit via Mayak Command Bridge'
        'comrades:Manage micro ecosystem tools (kvant and svyaz)'
        'check:Audit project compliance against GOST standard'
        'gost:Manage GOST synchronization, audit and upgrades'
        'serve:Unified HTTP server (static, dev with watcher, VDNKh demo)'
        'repo:Gosplan multi-repository aggregated management and overview'
        'act:Local CI runner, gate and structured log viewer'
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
        'post_push_sync:Combined list of post-push sync actions (tickets, agent)'
        'post_push_sync_tickets:Enable automatic tickets sync on post-push'
        'post_push_sync_agent:Enable automatic AGENTS.md sync on post-push'
        'post_push_sync_agents:Enable automatic AGENTS.md sync on post-push'
        'tickets_file:Destination file for Codeberg issues sync'
        'agent_md_url:URL or path to AGENTS.md skeleton template'
        'agents_md_url:URL or path to AGENTS.md skeleton template'
        'agent_md_file:Destination file for AGENTS.md directives'
        'agents_md_file:Destination file for AGENTS.md directives'
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
                ticket|issue|diva|divan)
                    _arguments \
                        '--dry-run[Simulate triage and preview tickets without forge publication]' \
                        '(-m --model)'{-m,--model}'[Gemini model name]:model:' \
                        '(-t --temp)'{-t,--temp}'[Sampling temperature]:temperature:' \
                        '(-r --repo)'{-r,--repo}'[Target repository slug]:repo:' \
                        '1:problem description or config:'
                    ;;
                agent|update-agent|sync-agent|agents|update-agents|sync-agents)
                    _arguments \
                        '(-u --url)'{-u,--url}'[Template URL or local file path]:url:_files' \
                        '(-f --file)'{-f,--file}'[Output markdown file]:file:_files' \
                        '1:subcommand:(update check fill set get unset)'
                    ;;
                codeberg)
                    _arguments \
                        '(-u --url)'{-u,--url}'[AGENTS template URL]:url:' \
                        '1:subcommand:(setup)'
                    ;;
                set-av)
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
                repo|repos)
                    _arguments \
                        '--dirty[List only repositories with uncommitted/unstaged changes]' \
                        '--unpushed[List only repositories with commits pending push]' \
                        '--issues[List only repositories with open tickets/issues]' \
                        '--tickets[List only repositories with open tickets/issues]' \
                        '--json[Output aggregated status as structured JSON]' \
                        '(-d --dir)'{-d,--dir}'[Target directory to scan for repositories]:directory:_directories' \
                        '(-c --catalog)'{-c,--catalog}'[Catalog file to use]:catalog file:_files' \
                        '(-f --fetch)'{-f,--fetch}'[Query remote forge for live issue counts]' \
                        '(-h --help)'{-h,--help}'[Display help message]' \
                        '1:subcommand:(status list overview help)'
                    ;;
                act)
                    _arguments \
                        '(-f --follow)'{-f,--follow}'[Follow log stream in real time]' \
                        '(-j --job)'{-j,--job}'[Target workflow job name]:job name:' \
                        '(-h --help)'{-h,--help}'[Display help message]' \
                        '1:subcommand:(log logs run status help)'
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
complete -c cccp -n "__fish_use_subcommand" -a tickets -d "Sync forge issues into markdown"
complete -c cccp -n "__fish_use_subcommand" -a sync-tickets -d "Sync forge issues into markdown"
complete -c cccp -n "__fish_use_subcommand" -a ticket -d "Interactive issue creation via Diva with GOST branch suggestion"
complete -c cccp -n "__fish_use_subcommand" -a issue -d "Interactive issue creation via Diva with GOST branch suggestion"
complete -c cccp -n "__fish_use_subcommand" -a diva -d "Gemini-powered issue triage bot with multi-forge dispatch"
complete -c cccp -n "__fish_use_subcommand" -a divan -d "Gemini-powered issue triage bot with multi-forge dispatch"
complete -c cccp -n "__fish_use_subcommand" -a catalog -d "Inventory workspaces, update description and sync forges"
complete -c cccp -n "__fish_use_subcommand" -a comrades -d "Manage micro ecosystem tools (kvant, svyaz, mayak)"
complete -c cccp -n "__fish_use_subcommand" -a check -d "Audit project compliance against GOST standard"
complete -c cccp -n "__fish_use_subcommand" -a gost -d "Manage GOST synchronization, audit and upgrades"
complete -c cccp -n "__fish_use_subcommand" -a serve -d "Unified HTTP server (static, dev with watcher, VDNKh demo)"
complete -c cccp -n "__fish_use_subcommand" -a repo -d "Gosplan multi-repository management and overview"
complete -c cccp -n "__fish_use_subcommand" -a act -d "Local CI runner, gate and structured log viewer"
complete -c cccp -n "__fish_use_subcommand" -a agent -d "Generate and synchronize AGENTS.md directives"
complete -c cccp -n "__fish_use_subcommand" -a update-agent -d "Generate and synchronize AGENTS.md directives"
complete -c cccp -n "__fish_use_subcommand" -a sync-agent -d "Generate and synchronize AGENTS.md directives"
complete -c cccp -n "__fish_use_subcommand" -a agents -d "Generate and synchronize AGENTS.md directives"
complete -c cccp -n "__fish_use_subcommand" -a update-agents -d "Generate and synchronize AGENTS.md directives"
complete -c cccp -n "__fish_use_subcommand" -a sync-agents -d "Generate and synchronize AGENTS.md directives"
complete -c cccp -n "__fish_use_subcommand" -a set-av -d "Fast shortcut to configure local agent variable"
complete -c cccp -n "__fish_use_subcommand" -a codeberg -d "Configure Codeberg integration and AGENTS template wizard"
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
complete -c cccp -n "__fish_seen_subcommand_from tickets sync-tickets" -s o -l owner -d "Repository owner"
complete -c cccp -n "__fish_seen_subcommand_from tickets sync-tickets" -s r -l repo -d "Repository name"
complete -c cccp -n "__fish_seen_subcommand_from tickets sync-tickets" -s p -l provider -d "Provider engine (forgejo, github, gitlab, sourcehut)"
complete -c cccp -n "__fish_seen_subcommand_from tickets sync-tickets" -s m -l max-bytes -d "Maximum budget in bytes"
complete -c cccp -n "__fish_seen_subcommand_from tickets sync-tickets" -s t -l token -d "Personal access token"

# Options for ticket and diva
complete -c cccp -n "__fish_seen_subcommand_from ticket issue diva divan" -l dry-run -d "Simulate triage without forge publication"
complete -c cccp -n "__fish_seen_subcommand_from ticket issue diva divan" -s m -l model -d "Gemini model name"
complete -c cccp -n "__fish_seen_subcommand_from ticket issue diva divan" -s t -l temp -d "Sampling temperature"
complete -c cccp -n "__fish_seen_subcommand_from ticket issue diva divan" -s r -l repo -d "Target repository slug"
complete -c cccp -n "__fish_seen_subcommand_from ticket issue diva divan" -a "config" -d "Configure Gemini API key"

# Options for agent
complete -c cccp -n "__fish_seen_subcommand_from agent update-agent sync-agent agents update-agents sync-agents" -s u -l url -d "Template URL or local file path"
complete -c cccp -n "__fish_seen_subcommand_from agent update-agent sync-agent agents update-agents sync-agents" -s f -l file -d "Output markdown file"
complete -c cccp -n "__fish_seen_subcommand_from agent update-agent sync-agent agents update-agents sync-agents" -a "update check fill set get unset"

# Options for codeberg
complete -c cccp -n "__fish_seen_subcommand_from codeberg" -a "setup" -d "Interactive Codeberg setup wizard"
complete -c cccp -n "__fish_seen_subcommand_from codeberg" -s u -l url -d "AGENTS template URL"

# Options for config
complete -c cccp -n "__fish_seen_subcommand_from config" -s g -l global -d "Target global config file"
complete -c cccp -n "__fish_seen_subcommand_from config" -l local -d "Target local repository config file"
complete -c cccp -n "__fish_seen_subcommand_from config" -s l -l list -d "List configuration key/value pairs"
complete -c cccp -n "__fish_seen_subcommand_from config" -l unset -d "Remove a configuration key"

# Options for diva
complete -c cccp -n "__fish_seen_subcommand_from diva divan" -s w -l web -d "Open interactive triage cockpit in web browser"
complete -c cccp -n "__fish_seen_subcommand_from diva divan" -l dry-run -d "Simulate triage without forge publication"
complete -c cccp -n "__fish_seen_subcommand_from diva divan" -s m -l model -d "Gemini model name"
complete -c cccp -n "__fish_seen_subcommand_from diva divan" -s t -l temp -d "Sampling temperature"
complete -c cccp -n "__fish_seen_subcommand_from diva divan" -s r -l repo -d "Target repository slug"
complete -c cccp -n "__fish_seen_subcommand_from diva divan" -a "config" -d "Persist GEMINI_API_KEY in .cccprc"

# Options for web
complete -c cccp -n "__fish_seen_subcommand_from web" -s p -l port -d "Port to listen on"
complete -c cccp -n "__fish_seen_subcommand_from web" -l host -d "Host address to bind to"
complete -c cccp -n "__fish_seen_subcommand_from web" -s o -l open -d "Open browser automatically"
complete -c cccp -n "__fish_seen_subcommand_from web" -s t -l token -d "API authentication token"
complete -c cccp -n "__fish_seen_subcommand_from web" -l no-open -d "Do not open browser"
complete -c cccp -n "__fish_seen_subcommand_from web" -s q -l quiet -d "Quiet mode"
complete -c cccp -n "__fish_seen_subcommand_from web" -a "diva status tickets"

# Options for repo
complete -c cccp -n "__fish_seen_subcommand_from repo" -l dirty -d "List only repositories with uncommitted/unstaged changes"
complete -c cccp -n "__fish_seen_subcommand_from repo" -l unpushed -d "List only repositories with commits pending push"
complete -c cccp -n "__fish_seen_subcommand_from repo" -l issues -d "List only repositories with open tickets"
complete -c cccp -n "__fish_seen_subcommand_from repo" -l json -d "Output aggregated status as JSON"
complete -c cccp -n "__fish_seen_subcommand_from repo" -s d -l dir -d "Target directory to scan for repositories"
complete -c cccp -n "__fish_seen_subcommand_from repo" -s c -l catalog -d "Catalog file to use"
complete -c cccp -n "__fish_seen_subcommand_from repo" -s f -l fetch -d "Query remote forge for live issue counts"
complete -c cccp -n "__fish_seen_subcommand_from repo" -a "status list overview help"

# Options for act
complete -c cccp -n "__fish_seen_subcommand_from act" -s f -l follow -d "Follow log stream in real time"
complete -c cccp -n "__fish_seen_subcommand_from act" -s j -l job -d "Target workflow job name"
complete -c cccp -n "__fish_seen_subcommand_from act" -a "log logs run status help"

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

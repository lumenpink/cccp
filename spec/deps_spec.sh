#!/bin/sh
# =============================================================================
# ShellSpec tests for scripts/check-deps.sh (Dependency Auditor)
# =============================================================================
# shellcheck shell=sh

Describe "check-deps.sh dependency auditor"

    SCRIPT="${PWD}/scripts/check-deps.sh"
    SRC_DIR="${PWD}/src"

    Describe "basic invocation"
        It "displays help on --help"
            When run sh "$SCRIPT" --help
            The status should be success
            The output should include "CCCP Dependency Auditor"
            The output should include "--list"
            The output should include "--verify"
            The output should include "--check"
        End

        It "exits 1 on unknown option"
            When run sh "$SCRIPT" --unknown-option
            The status should be failure
            The stderr should include "Unknown option"
        End

        It "exits 1 when target path does not exist"
            When run sh "$SCRIPT" /nonexistent/path
            The status should be failure
            The stderr should include "does not exist"
        End
    End

    Describe "--list mode"
        It "lists git as a mandatory external binary"
            When run sh "$SCRIPT" --list
            The status should be success
            The output should include "git"
        End

        It "lists optional external binaries found via command -v probes"
            When run sh "$SCRIPT" --list
            The status should be success
            The output should include "curl"
            The output should include "gh"
            The output should include "jq"
            The output should include "wget"
        End

        It "does not list internal project functions as external"
            When run sh "$SCRIPT" --list
            The status should be success
            The output should not include "calculate_target_version"
            The output should not include "load_hierarchical_config"
            The output should not include "show_help"
            The output should not include "generate_changelog"
        End
    End

    Describe "--check mode (default)"
        It "passes audit on clean src/ directory"
            When run sh "$SCRIPT" --check
            The status should be success
            The output should include "Dependency audit passed cleanly"
        End

        It "detects banned utility awk in a temporary test file"
            test_dir=$(mktemp -d)
            printf '#!/bin/sh\nresult=$(printf "%s" "$val" | awk "{ print $1 }")\n' > "${test_dir}/bad.sh"
            When run sh "$SCRIPT" --check "$test_dir"
            rm -rf "$test_dir"
            The status should be failure
            The output should include "Auditing codebase"
            The stderr should include "Banned utility detected: awk"
        End

        It "detects unauthorized binary python3 in a temporary test file"
            test_dir=$(mktemp -d)
            printf '#!/bin/sh\npython3 script.py\n' > "${test_dir}/bad.sh"
            When run sh "$SCRIPT" --check "$test_dir"
            rm -rf "$test_dir"
            The status should be failure
            The output should include "Auditing codebase"
            The stderr should include "Unauthorized binary detected: python3"
        End

        It "passes audit on a clean minimal script"
            test_dir=$(mktemp -d)
            printf '#!/bin/sh\ngrep -q "foo" file.txt\nsed -e "s/a/b/" file.txt\n' > "${test_dir}/clean.sh"
            When run sh "$SCRIPT" --check "$test_dir"
            rm -rf "$test_dir"
            The status should be success
            The output should include "Dependency audit passed cleanly"
        End
    End

    Describe "--verify mode"
        It "passes verification against DEPENDENCIES.md on clean src/"
            When run sh "$SCRIPT" --verify
            The status should be success
            The output should include "accurately documented"
        End

        It "fails verification when DEPENDENCIES.md is missing"
            test_dir=$(mktemp -d)
            fake_deps="/nonexistent/DEPENDENCIES.md"
            printf '#!/bin/sh\nif command -v curl >/dev/null 2>&1; then\n  curl -s https://example.com\nfi\n' > "${test_dir}/script.sh"
            When run env DEPS_FILE="$fake_deps" sh "$SCRIPT" --verify "$test_dir"
            rm -rf "$test_dir"
            The status should be failure
            The stderr should include "DEPENDENCIES.md does not exist"
        End

        It "fails when a dependency is used but not documented"
            test_dir=$(mktemp -d)
            fake_deps="${test_dir}/EMPTY.md"
            printf '# Empty\n' > "$fake_deps"
            printf '#!/bin/sh\nif command -v curl >/dev/null 2>&1; then\n  curl -s https://example.com\nfi\n' > "${test_dir}/script.sh"
            When run env DEPS_FILE="$fake_deps" sh "$SCRIPT" --verify "$test_dir"
            rm -rf "$test_dir"
            The status should be failure
            The output should include "Auditing external dependencies"
            The stderr should include "missing from"
        End
    End

    Describe "zero-awk compliance"
        It "verifies zero executable awk invocations in scripts/check-deps.sh itself"
            When run grep -E "(^|[[:space:]|;&(])awk([[:space:]|;&)]|$)" "${PWD}/scripts/check-deps.sh"
            The status should be failure
        End
    End

End

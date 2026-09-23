#!/bin/sh
# =============================================================================
# ShellSpec tests for scripts/lint.sh (POSIX Shell Linter)
# =============================================================================
# shellcheck shell=sh

Describe "scripts/lint.sh shell linter"

    SCRIPT="${PWD}/scripts/lint.sh"

    Describe "basic invocation"
        It "displays help on --help"
            When run sh "$SCRIPT" --help
            The status should be success
            The output should include "CCCP Shell Lint"
            The output should include "--syntax"
            The output should include "--shellcheck"
            The output should include "--fmt"
            The output should include "--fix"
        End

        It "exits 1 on unknown mode"
            When run sh "$SCRIPT" --bogus-mode
            The status should be failure
            The stderr should include "Unknown mode"
        End
    End

    Describe "--syntax mode"
        It "passes on clean src/ directory"
            When run sh "$SCRIPT" --syntax
            The status should be success
            The output should include "passed sh -n syntax check"
        End

        It "reports PASS for each script scanned"
            When run sh "$SCRIPT" --syntax
            The status should be success
            The output should include "[PASS]"
        End

        It "verifies all src/ scripts pass sh -n with zero errors"
            When run sh "$SCRIPT" --syntax
            The status should be success
            The output should not include "[FAIL]"
        End
    End

    Describe "--shellcheck mode (graceful degradation)"
        It "exits 0 and warns when shellcheck binary is not in PATH"
            # Use a minimal PATH that keeps /bin/sh but removes shellcheck
            _safe_path="/usr/bin:/bin"
            When run env "PATH=${_safe_path}" sh "$SCRIPT" --shellcheck
            The status should be success
            The stderr should include "shellcheck not found"
        End
    End

    Describe "--fmt mode (graceful degradation)"
        It "exits 0 and warns when shfmt binary is not in PATH"
            _safe_path="/usr/bin:/bin"
            When run env "PATH=${_safe_path}" sh "$SCRIPT" --fmt
            The status should be success
            The stderr should include "shfmt not found"
        End
    End

    Describe "--fix mode"
        It "exits 1 with actionable error when shfmt is not installed"
            _safe_path="/usr/bin:/bin"
            When run env "PATH=${_safe_path}" sh "$SCRIPT" --fix
            The status should be failure
            The stderr should include "shfmt not found"
        End
    End

    Describe "default (all) mode"
        It "passes when shellcheck and shfmt are absent"
            _safe_path="/usr/bin:/bin"
            When run env "PATH=${_safe_path}" sh "$SCRIPT"
            The status should be success
            The output should include "All checks passed"
            The stderr should include "shellcheck not found"
        End

        It "runs all three check stages in sequence"
            When run sh "$SCRIPT"
            The status should be success
            The output should include "[1/3] Syntax Check"
            The output should include "[2/3] Static Analysis"
            The output should include "[3/3] Format Compliance"
            The stderr should include "shellcheck not found"
        End
    End

    Describe "zero-awk compliance"
        It "verifies zero executable awk invocations in scripts/lint.sh itself"
            When run grep -E "(^|[[:space:]|;&(])awk([[:space:]|;&)]|$)" "${PWD}/scripts/lint.sh"
            The status should be failure
        End
    End

End

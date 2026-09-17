# Include the test helper
. "$SHELLSPEC_PROJECT_ROOT/spec/spec_helper.sh"

run_cli() {
  CHECK_UPDATES=0 CCCP_SOURCED=0 /bin/sh "$CCCP_BIN" "$@"
}

Describe 'CLI Entry Point (End-to-End)'
  setup() {
    TEST_DIR="$(mktemp -d)"
    cd "$TEST_DIR"
    git init
    git config user.email "test@example.com"
    git config user.name "Test User"
    touch README.md
    git add README.md
    git commit -m "chore: initial commit"
  }

  cleanup() {
    rm -rf "$TEST_DIR"
  }

  BeforeEach 'setup'
  AfterEach 'cleanup'

  Describe 'CLI dispatcher'
    It 'prints usage and exits with 1 when called without arguments'
      When call run_cli
      The status should be failure
      The output should include "Usage:"
      The output should include "help"
    End

    It 'prints usage and exits with 1 when called with unknown command'
      When run run_cli "non-existent-directive"
      The status should be failure
      The output should include "Usage:"
      The output should include "help"
    End

    It 'displays help on --help flag'
      When run run_cli --help
      The status should be success
      The output should include "Git Conventional Commits Helper Script"
    End

    It 'displays help on -h flag'
      When run run_cli -h
      The status should be success
      The output should include "Git Conventional Commits Helper Script"
    End

    It 'executes status command end-to-end'
      When run run_cli status
      The status should be success
      The output should include "CCCP State Inspection (Gosplan Quality Control)"
      The output should include "Active Branch:"
    End

    It 'executes version command end-to-end'
      When run run_cli version
      The status should be success
      The output should include "Version information written to VERSION file"
      The file "$TEST_DIR/VERSION" should be exist
    End

    It 'executes soviet easter egg command end-to-end'
      When run run_cli soviet
      The status should be success
      The output should include "Workers of the World, Conventionalize! ☭"
      The output should include "ORDER NO. 227"
    End

    It 'executes completion command end-to-end'
      When run run_cli completion bash
      The status should be success
      The output should include "# CCCP Bash Completion"
      The output should include "_cccp_completions()"
    End

    It 'executes check-update command end-to-end'
      When run run_cli check-update
      The status should be success
      The output should include "★ CCCP Update Verification Bureau ★"
      The output should include "Installed Version :"
      The output should include "Release Channel   :"
    End

    It 'executes hooks audit command end-to-end'
      When run run_cli hooks
      The status should be success
      The output should include "CCCP Git Hooks Inspectorate (Komissariat Audit)"
      The output should include "Hook: commit-msg"
      The output should include "Hook: post-commit"
    End
  End
End

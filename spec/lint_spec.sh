# Include the test helper
. "$SHELLSPEC_PROJECT_ROOT/spec/spec_helper.sh"

Describe 'lint'
  Include "$SHELLSPEC_PROJECT_ROOT/src/utils/lint.sh"
  Include "$SHELLSPEC_PROJECT_ROOT/src/utils/validation.sh"
  Include "$SHELLSPEC_PROJECT_ROOT/src/utils/config_manager.sh"

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

  Describe 'lint_commits'
    It 'passes when all commits in range follow Conventional Commits'
      echo "code" >> README.md
      git add README.md
      git commit -m "feat(api): add v1 endpoints"

      echo "docs" >> README.md
      git add README.md
      git commit -m "docs: document endpoints"

      When call lint_commits "HEAD~2..HEAD"
      The status should be success
      The output should include "★ CCCP Party Line Compliance (Commit Linting) ★"
      The output should include "✓"
      The output should include "PASSED: Full ideological compliance verified"
    End

    It 'fails when non-compliant commits exist in range'
      echo "code" >> README.md
      git add README.md
      git commit -m "feat: valid feature"

      echo "hack" >> README.md
      git add README.md
      git commit -m "fixed stuff without conventional prefix"

      When call lint_commits "HEAD~2..HEAD"
      The status should be failure
      The output should include "✗"
      The output should include "Order No. 227 - Not One Step Back from Clean Commits!"
      The output should include "FAILED: Non-compliant commits detected"
    End

    It 'handles empty commit range gracefully'
      When call lint_commits "HEAD..HEAD"
      The status should be success
      The output should include "0 commits found in range to inspect"
    End

    It 'rejects invalid git revision range'
      When call lint_commits "non_existent_branch..HEAD"
      The status should be failure
      The error should include "Error: Invalid commit range or revision"
    End

    It 'fails when executed outside of a git repository'
      NON_GIT_DIR="$(mktemp -d)"
      cd "$NON_GIT_DIR"
      When call lint_commits
      The status should be failure
      The error should include "Error: Not a git repository"
      rm -rf "$NON_GIT_DIR"
    End
  End
End

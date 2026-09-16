# Include the test helper
. "$SHELLSPEC_PROJECT_ROOT/spec/spec_helper.sh"

Describe 'post-commit'
  Include "${CCCP_BUNDLE:-$SHELLSPEC_PROJECT_ROOT/src/hooks/post_commit.sh}"
  Include "${CCCP_BUNDLE:-$SHELLSPEC_PROJECT_ROOT/src/utils/version.sh}"
  Include "${CCCP_BUNDLE:-$SHELLSPEC_PROJECT_ROOT/src/utils/changelog.sh}"

  setup() {
    TEST_DIR="$(mktemp -d)"
    cd "$TEST_DIR"
    git init
    git config user.email "test@example.com"
    git config user.name "Test User"
    touch README.md
    git add README.md
    git commit -m "initial commit"
  }

  cleanup() {
    rm -rf "$TEST_DIR"
  }

  BeforeEach 'setup'
  AfterEach 'cleanup'

  Describe 'post_commit hook'
    It 'generates VERSION and CHANGELOG.md and amends the commit'
      echo "feat: first feature" >> test.txt
      git add test.txt
      git commit -m "feat: first feature"
      
      When call post_commit
      The status should be success
      The output should include "Version information written to VERSION file"
      The file VERSION should be exist
      The file CHANGELOG.md should be exist
      The contents of file CHANGELOG.md should include "Features"
      The contents of file CHANGELOG.md should include "first feature"
    End

    It 'exits immediately when HOOK_ACTIVE is 1 to prevent infinite loop'
      export HOOK_ACTIVE=1
      When call post_commit
      The status should be success
      The file VERSION should not be exist
      unset HOOK_ACTIVE
    End

    It 'fails when executed outside of a git repository'
      NON_GIT_DIR="$(mktemp -d)"
      cd "$NON_GIT_DIR"
      When call post_commit
      The status should be failure
      The error should include "Error: Not a git repository"
      rm -rf "$NON_GIT_DIR"
    End
  End
End

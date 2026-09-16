# Include the test helper
. "$SHELLSPEC_PROJECT_ROOT/spec/spec_helper.sh"

Describe 'Commit'
  Include "${CCCP_BUNDLE:-$SHELLSPEC_PROJECT_ROOT/src/utils/commit.sh}"

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

  BeforeAll 'setup'
  AfterAll 'cleanup'

  Describe 'commit function'
    It 'returns error when no message is provided'
      When call commit ""
      The status should be failure
      The output should include "Usage:"
    End

    It 'validates commit message before committing'
      # Mock the validate_commit_message function to return failure
      validate_commit_message() { return 1; }
      
      When call commit "invalid message"
      The status should be failure
    End

    It 'calls git commit with the provided message when valid'
      # Mock the validate_commit_message function to return success
      validate_commit_message() { return 0; }
      
      # Mock the git command
      git() { echo "called git with: $*"; }

      When call commit "feat: valid message"
      The status should be success
      The output should include "called git with: commit -m feat: valid message"
    End

    It 'handles conventional commit format with scope'
      # Mock the validate_commit_message function to return success
      validate_commit_message() { return 0; }
      
      # Mock the git command
      git() { echo "called git with: $*"; }

      When call commit "feat(ui): add button"
      The status should be success
      The output should include "called git with: commit -m feat(ui): add button"
    End

    It 'handles conventional commit format with multiple scopes'
      # Mock the validate_commit_message function to return success
      validate_commit_message() { return 0; }
      
      # Mock the git command
      git() { echo "called git with: $*"; }

      When call commit "feat(ui,api): add login"
      The status should be success
      The output should include "called git with: commit -m feat(ui,api): add login"
    End

    It 'handles conventional commit format with subscopes'
      # Mock the validate_commit_message function to return success
      validate_commit_message() { return 0; }
      
      # Mock the git command
      git() { echo "called git with: $*"; }

      When call commit "feat(ui/components): add button"
      The status should be success
      The output should include "called git with: commit -m feat(ui/components): add button"
    End

    It 'displays help information on -h or --help'
      When call commit -h
      The status should be success
      The output should include "commit"
    End

    It 'rejects unexpected trailing arguments'
      When call commit "feat: message" "unexpected_arg"
      The status should be failure
      The error should include "Error: Unexpected argument 'unexpected_arg'"
    End

    It 'passes flag options to override scope rules'
      # Mock validate_commit_message to record global flags
      validate_commit_message() {
        echo "SCOPES=$STRICT_SCOPES SUBSCOPES=$DISABLE_SUBSCOPES MULTI=$DISABLE_MULTIPLE_SCOPES"
        return 0
      }
      # Preserve git rev-parse for git root detection while stubbing commit
      git() {
        if [ "$1" = "rev-parse" ]; then
          command git "$@"
        fi
      }

      When call commit --strict-scopes --disable-subscopes --disable-multiple-scopes "feat: message"
      The status should be success
      The output should include "SCOPES=1 SUBSCOPES=1 MULTI=1"
    End

    It 'fails when executed outside of a git repository'
      NON_GIT_DIR="$(mktemp -d)"
      ORIG_DIR="$(pwd)"
      cd "$NON_GIT_DIR"
      When call commit "feat: outside"
      The status should be failure
      The error should include "Error: Not a git repository"
      cd "$ORIG_DIR"
      rm -rf "$NON_GIT_DIR"
    End
  End
End
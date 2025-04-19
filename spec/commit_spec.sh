# Include the test helper
. "$SHELLSPEC_PROJECT_ROOT/spec/spec_helper.sh"

Describe 'Commit'
  Include "$SHELLSPEC_PROJECT_ROOT/src/utils/commit.sh"

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
  End
End
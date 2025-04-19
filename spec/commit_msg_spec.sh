# Include the test helper
. "$SHELLSPEC_PROJECT_ROOT/spec/spec_helper.sh"

Describe 'commit-msg hook'
  Include "$SHELLSPEC_PROJECT_ROOT/src/utils/validation.sh"
  Include "$SHELLSPEC_PROJECT_ROOT/src/hooks/commit_msg.sh"

  setup() {
    # Create a temporary directory for the test repository
    TEST_DIR="$(mktemp -d)"
    cd "$TEST_DIR"
    
    # Initialize git repository
    git init
    git config user.email "test@example.com"
    git config user.name "Test User"
    
    # Create a temporary commit message file
    COMMIT_MSG_FILE="$(mktemp)"
    
    # Create a mock config file
    mkdir -p "$TEST_DIR/src/config"
    cat > "$TEST_DIR/src/config/config.sh" << 'EOF'
validate_commit_message() {
  local message="$1"
  # Mock validation - accept any non-empty message
  [ -n "$message" ]
}
EOF
  }

  cleanup() {
    # Clean up the temporary directory and files
    rm -rf "$TEST_DIR"
    rm -f "$COMMIT_MSG_FILE"
  }

  BeforeAll 'setup'
  AfterAll 'cleanup'

  Describe 'commit_msg function'
    It 'validates a valid commit message'
      echo "feat: add new feature" > "$COMMIT_MSG_FILE"
      When call commit_msg "$COMMIT_MSG_FILE"
      The status should be success
    End

    It 'rejects an empty commit message'
      echo "" > "$COMMIT_MSG_FILE"
      When call commit_msg "$COMMIT_MSG_FILE"
      The status should be failure
      The output should include "Error: Commit message can't be empty"
      The stderr should include "Error: Invalid commit message"
    End

    It 'handles non-existent message file'
      When call commit_msg "/nonexistent/file"
      The status should be failure
      The stderr should include "Error: Failed to read commit message"
    End

    It 'validates a multi-line commit message'
      cat > "$COMMIT_MSG_FILE" << 'EOF'
feat: add new feature

This is a detailed description of the feature.
It spans multiple lines.

- Added new functionality
- Fixed some bugs
EOF
      When call commit_msg "$COMMIT_MSG_FILE"
      The status should be success
    End

    It 'handles special characters in commit message'
      echo "fix: handle special chars: \`~!@#$%^&*()_+-=[]{}|;:',.<>?" > "$COMMIT_MSG_FILE"
      When call commit_msg "$COMMIT_MSG_FILE"
      The status should be success
    End
  End
End 
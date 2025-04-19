# Include the test helper
. "$SHELLSPEC_PROJECT_ROOT/spec/spec_helper.sh"

Describe 'hooks'
  Include "$SHELLSPEC_PROJECT_ROOT/src/hooks/hooks.sh"

  setup() {
    # Create a temporary directory for the test repository
    TEST_DIR="$(mktemp -d)"
    cd "$TEST_DIR"
    
    # Initialize git repository
    git init
    git config user.email "test@example.com"
    git config user.name "Test User"
    
    # Set GIT_ROOT for testing
    GIT_ROOT="$TEST_DIR"
    GIT_HOOKS_DIR="$GIT_ROOT/.git/hooks"
    echo "echo 'test'" > $GIT_HOOK_FILE
    chmod +x $GIT_HOOK_FILE
  }

  cleanup() {
    # Clean up the temporary directory
    rm -rf "$TEST_DIR"
  }

  BeforeAll 'setup'
  AfterAll 'cleanup'

  Describe 'install_git_hooks'
    BeforeEach 'mkdir -p $GIT_HOOKS_DIR && rm -f $GIT_HOOKS_DIR/commit-msg $GIT_HOOKS_DIR/commit-msg.old'

    It 'creates hooks directory'
      When call install_git_hooks
      The output should include "Successfully installed git hooks!"
      The path "$GIT_HOOKS_DIR" should be exist
    End

    It 'creates symlinks for all hooks'
      When call install_git_hooks
      The output should include "Successfully installed git hooks!"
      for hook in $GIT_HOOKS_LIST; do
        The path "$GIT_HOOKS_DIR/$hook" should be symlink
      done
    End

    It 'backs up existing hooks'
      # Create a test hook
      echo "test" > "$GIT_HOOKS_DIR/commit-msg"
      When call install_git_hooks
      The output should include "Backed up existing hook"
      The path "$GIT_HOOKS_DIR/commit-msg.old" should be exist
      The contents of file "$GIT_HOOKS_DIR/commit-msg.old" should include "test"
    End

    It 'skips already installed hooks'
      # Create a symlink
      ln -s "$GIT_ROOT/sh-cc-commits.sh" "$GIT_HOOKS_DIR/commit-msg"
      When call install_git_hooks
      The output should include "Hook already installed"
    End

    It 'replaces symlinks pointing to other files'
      # Create a different file
      echo "echo 'other script'" > other-script.sh
      chmod +x other-script.sh
      
      # Create a symlink to the other file
      ln -s "$GIT_ROOT/other-script.sh" "$GIT_HOOKS_DIR/commit-msg"
      
      # Run install hooks
      When call install_git_hooks
      The output should include "Backed up existing hook"
      # Verify the symlink now points to our script
      The path "$GIT_HOOKS_DIR/commit-msg" should be symlink
      The value "$(readlink "$GIT_HOOKS_DIR/commit-msg")" should equal "$GIT_ROOT/$GIT_HOOK_FILE"
      
      # Verify the old symlink was backed up
      The path "$GIT_HOOKS_DIR/commit-msg.old" should be symlink
      The value "$(readlink "$GIT_HOOKS_DIR/commit-msg.old")" should equal "$GIT_ROOT/other-script.sh"
    End
    
    It 'backs up regular files with incrementing suffixes'
      # First create a hook file
      echo "bogus content" > "$GIT_HOOKS_DIR/commit-msg"
      
      # Run install hooks
      When call install_git_hooks
      The output should include "Backed up existing hook"
      
      # Verify the new hook is a symlink to our script
      The path "$GIT_HOOKS_DIR/commit-msg" should be symlink
      The value "$(readlink "$GIT_HOOKS_DIR/commit-msg")" should equal "$GIT_ROOT/$GIT_HOOK_FILE"
      
      # Verify the old file was backed up
      The path "$GIT_HOOKS_DIR/commit-msg.old" should be exist
      The contents of file "$GIT_HOOKS_DIR/commit-msg.old" should equal "bogus content"
    End

    # It 'configures git to use local hooks'
    #   When call install_git_hooks
    #   The output of "git config core.hooksPath" should equal "$GIT_HOOKS_DIR"
    # End
  End
End 
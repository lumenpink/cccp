# Include the test helper
. "$SHELLSPEC_PROJECT_ROOT/spec/spec_helper.sh"

Describe 'hooks'
  Include "$SHELLSPEC_PROJECT_ROOT/src/hooks/hooks.sh"

  setup() {
    TEST_DIR="$(mktemp -d)"
    cd "$TEST_DIR"
    
    git init
    git config user.email "test@example.com"
    git config user.name "Test User"
    
    GIT_ROOT="$TEST_DIR"
    GIT_HOOKS_DIR="$GIT_ROOT/.git/hooks"
    echo "1.2.0" > "$GIT_ROOT/VERSION"
  }

  cleanup() {
    rm -rf "$TEST_DIR"
  }

  BeforeAll 'setup'
  AfterAll 'cleanup'

  Describe 'install_git_hooks'
    BeforeEach 'mkdir -p "$GIT_HOOKS_DIR" && rm -f "$GIT_HOOKS_DIR/commit-msg"* "$GIT_HOOKS_DIR/post-commit"*'

    It 'creates hooks directory'
      When call install_git_hooks
      The output should include "Successfully installed git hooks!"
      The path "$GIT_HOOKS_DIR" should be exist
    End

    It 'creates portable wrapper scripts for all hooks'
      When call install_git_hooks
      The output should include "Successfully installed git hooks!"
      The output should include "Hooks configured with cccp version: 1.2.0"
      for hook in $GIT_HOOKS_LIST; do
        The path "$GIT_HOOKS_DIR/$hook" should be file
        The contents of file "$GIT_HOOKS_DIR/$hook" should include "# cccp-hook-version: 1.2.0"
        The contents of file "$GIT_HOOKS_DIR/$hook" should include "exec cccp $hook"
      done
    End

    It 'backs up existing hooks'
      echo "custom hook" > "$GIT_HOOKS_DIR/commit-msg"
      When call install_git_hooks
      The output should include "Backed up existing hook"
      The path "$GIT_HOOKS_DIR/commit-msg.old" should be exist
      The contents of file "$GIT_HOOKS_DIR/commit-msg.old" should include "custom hook"
    End

    It 'skips already installed up-to-date hooks'
      install_git_hooks >/dev/null 2>&1
      When call install_git_hooks
      The output should include "Hook already installed"
    End

    It 'backs up files with incrementing suffixes'
      echo "first backup" > "$GIT_HOOKS_DIR/commit-msg.old"
      echo "active hook" > "$GIT_HOOKS_DIR/commit-msg"
      
      When call install_git_hooks
      The output should include "Backed up existing hook"
      The path "$GIT_HOOKS_DIR/commit-msg.old" should be exist
      The contents of file "$GIT_HOOKS_DIR/commit-msg.old" should include "first backup"
      The path "$GIT_HOOKS_DIR/commit-msg.old.1" should be exist
      The contents of file "$GIT_HOOKS_DIR/commit-msg.old.1" should include "active hook"
    End
  End
End
# Include the test helper
. "$SHELLSPEC_PROJECT_ROOT/spec/spec_helper.sh"

Describe 'hooks'
  Include "${CCCP_BUNDLE:-$SHELLSPEC_PROJECT_ROOT/src/hooks/hooks.sh}"

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

  Describe 'install_global_binary'
    setup_global() {
      MOCK_BIN_DIR="$(mktemp -d)"
      MOCK_HOME="$(mktemp -d)"
      export XDG_BIN_HOME="$MOCK_BIN_DIR"
      export HOME="$MOCK_HOME"
      export PATH="/usr/bin:/bin"
      echo "#!/bin/sh" > "$GIT_ROOT/cccp.sh"
      echo "echo 'mock cccp'" >> "$GIT_ROOT/cccp.sh"
      chmod +x "$GIT_ROOT/cccp.sh"
    }

    cleanup_global() {
      rm -rf "$MOCK_BIN_DIR" "$MOCK_HOME"
    }

    BeforeEach 'setup_global'
    AfterEach 'cleanup_global'

    It 'installs binary to target directory and makes it executable'
      When call install_global_binary "$GIT_ROOT/cccp.sh"
      The status should be success
      The output should include "Successfully installed cccp to: $MOCK_BIN_DIR/cccp"
      The path "$MOCK_BIN_DIR/cccp" should be executable
    End

    It 'adds target directory to shell profile when not in PATH'
      When call install_global_binary "$GIT_ROOT/cccp.sh"
      The output should include "is not currently in your \$PATH"
      The path "$MOCK_HOME/.bashrc" should be file
      The contents of file "$MOCK_HOME/.bashrc" should include "$MOCK_BIN_DIR"
    End

    It 'skips profile update when target directory is already in PATH'
      export PATH="$MOCK_BIN_DIR:$PATH"
      When call install_global_binary "$GIT_ROOT/cccp.sh"
      The output should include "You can now run 'cccp' from anywhere!"
      The path "$MOCK_HOME/.bashrc" should not be exist
    End
  End

  Describe 'install_cccp CLI dispatcher'
    It 'dispatches to global installation with -g flag'
      MOCK_BIN_DIR="$(mktemp -d)"
      MOCK_HOME="$(mktemp -d)"
      export XDG_BIN_HOME="$MOCK_BIN_DIR"
      export HOME="$MOCK_HOME"
      echo "#!/bin/sh" > "$GIT_ROOT/cccp.sh"

      When call install_cccp -g
      The status should be success
      The output should include "Successfully installed cccp to:"
      rm -rf "$MOCK_BIN_DIR" "$MOCK_HOME"
    End

    It 'dispatches to local hook installation with no flags'
      When call install_cccp
      The status should be success
      The output should include "Successfully installed git hooks!"
    End
  End
End
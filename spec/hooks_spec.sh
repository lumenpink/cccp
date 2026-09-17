# Include the test helper
. "$SHELLSPEC_PROJECT_ROOT/spec/spec_helper.sh"

Describe 'hooks'
  Include "${CCCP_BUNDLE:-$SHELLSPEC_PROJECT_ROOT/src/utils/help.sh}"
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
      The output should include "Hooks configured with cccp version: ${CCCP_VERSION:-2.0.0}"
      for hook in $GIT_HOOKS_LIST; do
        The path "$GIT_HOOKS_DIR/$hook" should be file
        The contents of file "$GIT_HOOKS_DIR/$hook" should include "# cccp-hook-version: ${CCCP_VERSION:-2.0.0}"
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

  Describe 'audit_git_hooks'
    It 'reports missing hooks when none exist'
      rm -f "$GIT_HOOKS_DIR/commit-msg" "$GIT_HOOKS_DIR/post-commit"
      When call audit_git_hooks
      The status should be success
      The output should include "Hook: commit-msg"
      The output should include "Status      : Missing / Not installed"
      The output should include "Intervention: Run 'cccp install'"
    End

    It 'reports synchronized when hook matches active CCCP version'
      install_git_hooks
      When call audit_git_hooks
      The status should be success
      The output should include "Hook: commit-msg"
      The output should include "Status      : Synchronized"
      The output should include "Hook: post-commit"
      The output should include "Status      : Synchronized"
    End

    It 'reports outdated CCCP wrapper and recommends cccp install'
      echo "#!/bin/sh" > "$GIT_HOOKS_DIR/commit-msg"
      echo "# cccp-hook-version: 1.0.0" >> "$GIT_HOOKS_DIR/commit-msg"
      When call audit_git_hooks
      The status should be success
      The output should include "Status      : Outdated CCCP Wrapper (v1.0.0 vs current v"
      The output should include "Intervention: Run 'cccp install' to upgrade wrapper"
    End

    It 'detects custom hook, identifies Husky framework, and shows backups'
      echo "#!/bin/sh" > "$GIT_HOOKS_DIR/commit-msg"
      echo ". \"\$(dirname -- \"\$0\")/_/husky.sh\"" >> "$GIT_HOOKS_DIR/commit-msg"
      echo "npx --no-install commitlint" >> "$GIT_HOOKS_DIR/commit-msg"
      echo "old backup 1" > "$GIT_HOOKS_DIR/commit-msg.old"
      echo "old backup 2" > "$GIT_HOOKS_DIR/commit-msg.old.1"

      When call audit_git_hooks
      The status should be success
      The output should include "Status      : Custom / Non-CCCP"
      The output should include "Framework   : Husky"
      The output should include "Chains CCCP : No"
      The output should include "Backups     : commit-msg.old commit-msg.old.1"
      The output should include "To replace with CCCP: Run 'cccp install'"
      The output should include "To chain CCCP inside this hook: Add 'cccp commit-msg"
    End
  End

  Describe 'diff_git_hooks'
    It 'reports no differences when installed hook is identical to canonical wrapper'
      install_git_hooks
      When call diff_git_hooks "commit-msg"
      The status should be success
      The output should include "No differences found. Installed hook is identical to canonical CCCP wrapper."
    End

    It 'generates unified diff and intervention advice for custom hooks'
      echo "#!/bin/sh" > "$GIT_HOOKS_DIR/commit-msg"
      echo "echo 'custom non-cccp logic'" >> "$GIT_HOOKS_DIR/commit-msg"

      When call diff_git_hooks "commit-msg"
      The status should be success
      The output should include "=== Diff: commit-msg"
      The output should include "custom non-cccp logic"
      The output should include "cccp-hook-version:"
      The output should include "Intervention Guidance:"
      The output should include "Installed hook is custom/non-CCCP."
    End
  End

  Describe 'cmd_hooks dispatcher'
    It 'dispatches to audit by default'
      When call cmd_hooks
      The status should be success
      The output should include "CCCP Git Hooks Inspectorate (Komissariat Audit)"
    End

    It 'dispatches to diff on diff argument'
      install_git_hooks
      When call cmd_hooks diff commit-msg
      The status should be success
      The output should include "=== Diff: commit-msg"
      The output should include "No differences found."
    End

    It 'shows usage on --help flag'
      When call cmd_hooks --help
      The status should be success
      The output should include "cccp.sh hooks - Git Hooks Inspectorate and Diffing"
    End
  End

  reset_git() {
    cd "$TEST_DIR"
    export HOOK_ACTIVE=1
    echo "1.2.0" > "$TEST_DIR/VERSION"
    git add "$TEST_DIR/VERSION"
    git commit -m "chore: reset environment" >/dev/null 2>&1 || true
    unset HOOK_ACTIVE
    unset CCCP_TAG_ACTIVE || true
  }

  Describe 'cmd_pre_push'
    BeforeEach 'reset_git'

    It 'allows pushing normal branches without tags'
      test_push_branch() {
        echo "refs/heads/main 1111111111111111111111111111111111111111 refs/heads/main 0000000000000000000000000000000000000000" | cmd_pre_push "origin" "git@github.com:test/repo.git"
      }
      When call test_push_branch
      The status should be success
    End

    It 'allows pushing valid release tag with synchronized VERSION'
      echo "2.1.0" > "$TEST_DIR/VERSION"
      git add VERSION
      HOOK_ACTIVE=1 git commit -m "chore(release): v2.1.0"
      sha=$(git rev-parse HEAD)
      test_push_valid_tag() {
        echo "refs/tags/v2.1.0 $sha refs/tags/v2.1.0 0000000000000000000000000000000000000000" | cmd_pre_push "origin" "git@github.com:test/repo.git"
      }
      When call test_push_valid_tag
      The status should be success
    End

    It 'rejects pushing tag when commit has development VERSION'
      echo "2.1.0-dev.1+20260917.abc" > "$TEST_DIR/VERSION"
      git add VERSION
      HOOK_ACTIVE=1 git commit -m "feat: new feature"
      sha=$(git rev-parse HEAD)
      test_push_dev_tag() {
        echo "refs/tags/v2.1.0 $sha refs/tags/v2.1.0 0000000000000000000000000000000000000000" | cmd_pre_push "origin" "git@github.com:test/repo.git"
      }
      When call test_push_dev_tag
      The status should be failure
      The stderr should include "Error: Tag 'v2.1.0' points to a development version"
    End

    It 'rejects pushing tag when commit has missing VERSION file'
      rm -f "$TEST_DIR/VERSION"
      git rm -f VERSION 2>/dev/null || true
      touch "$TEST_DIR/dummy.txt"
      git add dummy.txt
      HOOK_ACTIVE=1 git commit -m "feat: commit without VERSION"
      sha=$(git rev-parse HEAD)
      test_push_missing_ver() {
        echo "refs/tags/v2.1.0 $sha refs/tags/v2.1.0 0000000000000000000000000000000000000000" | cmd_pre_push "origin" "git@github.com:test/repo.git"
      }
      When call test_push_missing_ver
      The status should be failure
      The stderr should include "Error: Tag 'v2.1.0' points to a commit without a VERSION file"
    End

    It 'rejects pushing tag when commit has mismatched VERSION'
      echo "2.0.0" > "$TEST_DIR/VERSION"
      git add VERSION
      HOOK_ACTIVE=1 git commit -m "chore(release): v2.0.0"
      sha=$(git rev-parse HEAD)
      test_push_mismatched_ver() {
        echo "refs/tags/v2.1.0 $sha refs/tags/v2.1.0 0000000000000000000000000000000000000000" | cmd_pre_push "origin" "git@github.com:test/repo.git"
      }
      When call test_push_mismatched_ver
      The status should be failure
      The stderr should include "Error: Tag 'v2.1.0' does not match VERSION file in target commit"
    End

    It 'allows deleting remote tags'
      test_push_delete_tag() {
        echo "(delete) 0000000000000000000000000000000000000000 refs/tags/v2.1.0 1111111111111111111111111111111111111111" | cmd_pre_push "origin" "git@github.com:test/repo.git"
      }
      When call test_push_delete_tag
      The status should be success
    End
  End

  Describe 'cmd_reference_transaction'
    BeforeEach 'reset_git'

    It 'allows tag creation when CCCP_TAG_ACTIVE is 1'
      export CCCP_TAG_ACTIVE=1
      sha=$(git rev-parse HEAD)
      test_ref_tx_active() {
        echo "0000000000000000000000000000000000000000 $sha refs/tags/v2.1.0" | cmd_reference_transaction "prepared"
      }
      When call test_ref_tx_active
      The status should be success
      unset CCCP_TAG_ACTIVE || true
    End

    It 'allows tag creation when target commit has clean matching VERSION'
      echo "2.1.0" > "$TEST_DIR/VERSION"
      git add VERSION
      HOOK_ACTIVE=1 git commit -m "chore(release): v2.1.0"
      sha=$(git rev-parse HEAD)
      test_ref_tx_valid() {
        echo "0000000000000000000000000000000000000000 $sha refs/tags/v2.1.0" | cmd_reference_transaction "prepared"
      }
      When call test_ref_tx_valid
      The status should be success
    End

    It 'blocks tag creation when target commit has development VERSION'
      echo "2.1.0-dev.1+20260917.abc" > "$TEST_DIR/VERSION"
      git add VERSION
      HOOK_ACTIVE=1 git commit -m "feat: dev commit"
      sha=$(git rev-parse HEAD)
      test_ref_tx_dev() {
        echo "0000000000000000000000000000000000000000 $sha refs/tags/v2.1.0" | cmd_reference_transaction "prepared"
      }
      When call test_ref_tx_dev
      The status should be failure
      The stderr should include "Error: Cannot create tag 'v2.1.0'."
      The stderr should include "The target commit has a development VERSION"
    End

    It 'blocks tag creation when target commit has missing VERSION file'
      rm -f "$TEST_DIR/VERSION"
      git rm -f VERSION 2>/dev/null || true
      touch "$TEST_DIR/dummy2.txt"
      git add dummy2.txt
      HOOK_ACTIVE=1 git commit -m "feat: commit without VERSION"
      sha=$(git rev-parse HEAD)
      test_ref_tx_missing() {
        echo "0000000000000000000000000000000000000000 $sha refs/tags/v2.1.0" | cmd_reference_transaction "prepared"
      }
      When call test_ref_tx_missing
      The status should be failure
      The stderr should include "Error: Cannot create tag 'v2.1.0'."
      The stderr should include "The target commit ($sha) does not have a VERSION file."
    End

    It 'blocks tag creation when target commit has mismatched VERSION'
      echo "2.0.0" > "$TEST_DIR/VERSION"
      git add VERSION
      HOOK_ACTIVE=1 git commit -m "chore(release): v2.0.0"
      sha=$(git rev-parse HEAD)
      test_ref_tx_mismatch() {
        echo "0000000000000000000000000000000000000000 $sha refs/tags/v2.1.0" | cmd_reference_transaction "prepared"
      }
      When call test_ref_tx_mismatch
      The status should be failure
      The stderr should include "Error: Cannot create tag 'v2.1.0'."
      The stderr should include "Tag version does not match target commit VERSION"
    End

    It 'allows tag deletion'
      test_ref_tx_delete() {
        echo "1111111111111111111111111111111111111111 0000000000000000000000000000000000000000 refs/tags/v2.1.0" | cmd_reference_transaction "prepared"
      }
      When call test_ref_tx_delete
      The status should be success
    End

    It 'ignores non-prepared states'
      test_ref_tx_committed() {
        echo "0000000000000000000000000000000000000000 1111111111111111111111111111111111111111 refs/tags/v2.1.0" | cmd_reference_transaction "committed"
      }
      When call test_ref_tx_committed
      The status should be success
    End
  End
End
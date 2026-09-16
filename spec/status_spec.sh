# Include the test helper
. "$SHELLSPEC_PROJECT_ROOT/spec/spec_helper.sh"

Describe 'status'
  Include "${CCCP_BUNDLE:-$SHELLSPEC_PROJECT_ROOT/src/utils/status.sh}"
  Include "${CCCP_BUNDLE:-$SHELLSPEC_PROJECT_ROOT/src/utils/version.sh}"
  Include "${CCCP_BUNDLE:-$SHELLSPEC_PROJECT_ROOT/src/utils/config_manager.sh}"

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

  Describe 'show_status'
    It 'displays Gosplan State Inspection header and branch info'
      When call show_status
      The status should be success
      The output should include "CCCP State Inspection (Gosplan Quality Control)"
      The output should include "Repository:"
      The output should include "Active Branch:"
    End

    It 'reports clean working tree when no uncommitted changes exist'
      When call show_status
      The status should be success
      The output should include "Working Tree:     clean"
    End

    It 'reports dirty working tree with untracked files'
      touch untracked.txt
      When call show_status
      The status should be success
      The output should include "Working Tree:     dirty (1 untracked)"
    End

    It 'reports dirty working tree with modified and staged files'
      echo "change" >> README.md
      touch staged.txt
      git add staged.txt
      When call show_status
      The status should be success
      The output should include "1 staged"
      The output should include "1 modified"
    End

    It 'detects local configuration file when .cccprc exists'
      echo "strict_types = 1" > "$TEST_DIR/.cccprc"
      When call show_status
      The status should be success
      The output should include ".cccprc"
    End

    It 'detects installed cccp git hooks'
      mkdir -p "$TEST_DIR/.git/hooks"
      echo '#!/bin/sh' > "$TEST_DIR/.git/hooks/commit-msg"
      echo 'CCCP_HOOK_VERSION="1.3.0"' >> "$TEST_DIR/.git/hooks/commit-msg"
      touch "$TEST_DIR/.git/hooks/post-commit"
      When call show_status
      The status should be success
      The output should include "Git Hooks:        installed (v1.3.0)"
    End

    It 'displays permissive types policy and strict scopes policy when configured'
      export STRICT_TYPES="0"
      export STRICT_SCOPES="1"
      When call show_status
      The status should be success
      The output should include "Types Policy:     permissive (any alphanumeric)"
      The output should include "Scopes Policy:    strict (curated)"
      unset STRICT_TYPES STRICT_SCOPES || true
    End

    It 'reports up to date when branch matches upstream'
      REMOTE_DIR="$(mktemp -d)"
      git clone --bare "$TEST_DIR" "$REMOTE_DIR/remote.git" >/dev/null 2>&1
      git remote add origin "$REMOTE_DIR/remote.git"
      git fetch origin >/dev/null 2>&1
      git branch --set-upstream-to=origin/main main >/dev/null 2>&1 || git branch --set-upstream-to=origin/master master >/dev/null 2>&1

      show_status_tracking() { show_status | grep "Active Branch:"; }
      When call show_status_tracking
      The status should be success
      The output should include "up to date with origin/"
      rm -rf "$REMOTE_DIR"
    End

    It 'reports ahead of upstream when local commits are unpushed'
      REMOTE_DIR="$(mktemp -d)"
      git clone --bare "$TEST_DIR" "$REMOTE_DIR/remote.git" >/dev/null 2>&1
      git remote add origin "$REMOTE_DIR/remote.git"
      git fetch origin >/dev/null 2>&1
      git branch --set-upstream-to=origin/main main >/dev/null 2>&1 || git branch --set-upstream-to=origin/master master >/dev/null 2>&1

      echo "ahead work" >> README.md
      git commit -am "feat: ahead commit" >/dev/null 2>&1

      show_status_tracking() { show_status | grep "Active Branch:"; }
      When call show_status_tracking
      The status should be success
      The output should include "ahead of origin/"
      The output should include "by 1 commit(s)"
      rm -rf "$REMOTE_DIR"
    End

    It 'fails when executed outside of a git repository'
      NON_GIT_DIR="$(mktemp -d)"
      cd "$NON_GIT_DIR"
      When call show_status
      The status should be failure
      The error should include "Error: Not a git repository"
      rm -rf "$NON_GIT_DIR"
    End
  End
End

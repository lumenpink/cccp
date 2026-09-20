# Include the test helper
. "$SHELLSPEC_PROJECT_ROOT/spec/spec_helper.sh"

Describe 'push'
  Include "${CCCP_BUNDLE:-$SHELLSPEC_PROJECT_ROOT/src/utils/help.sh}"
  Include "${CCCP_BUNDLE:-$SHELLSPEC_PROJECT_ROOT/src/utils/config_manager.sh}"
  Include "${CCCP_BUNDLE:-$SHELLSPEC_PROJECT_ROOT/src/hooks/hooks.sh}"
  Include "${CCCP_BUNDLE:-$SHELLSPEC_PROJECT_ROOT/src/utils/push.sh}"

  setup() {
    TEST_DIR="$(mktemp -d)"
    cd "$TEST_DIR"

    git init
    git config user.email "test@example.com"
    git config user.name "Test User"
    echo "1.0.0" > "$TEST_DIR/VERSION"
    git add VERSION
    git commit -m "chore: initial commit"

    GIT_ROOT="$TEST_DIR"
    GIT_HOOKS_DIR="$GIT_ROOT/.git/hooks"
    export POST_PUSH_SYNC=""
    export POST_PUSH_SYNC_TICKETS="0"
    export POST_PUSH_SYNC_AGENT="0"
    export POST_PUSH_SYNC_AGENTS="0"
  }

  cleanup() {
    rm -rf "$TEST_DIR"
  }

  BeforeEach 'setup'
  AfterEach 'cleanup'

  Describe 'cmd_push'
    It 'shows usage on --help flag'
      When call cmd_push --help
      The status should be success
      The output should include "cccp.sh push - Execute Git Push and Trigger Optional Post-Push Hook"
      The output should include "Usage:"
    End

    It 'executes git push and triggers post-push hook on success'
      REMOTE_DIR="$(mktemp -d)"
      git clone --bare "$TEST_DIR" "$REMOTE_DIR/remote.git" >/dev/null 2>&1
      git remote add origin "$REMOTE_DIR/remote.git"
      git branch -M main

      echo "new line" >> README.md
      git add README.md
      git commit -m "feat: new feature"

      log_file="$(mktemp)"
      script_file="$(mktemp)"
      echo '#!/bin/sh' > "$script_file"
      echo 'echo "TRIGGERED: $1 $2" >> "'"$log_file"'"' >> "$script_file"
      chmod +x "$script_file"
      export POST_PUSH_HOOK="$script_file"

      test_cmd_push_success() {
        cmd_push -u origin main >/dev/null 2>&1
      }
      When call test_cmd_push_success
      The status should be success
      The contents of file "$log_file" should include "TRIGGERED: origin main"

      rm -rf "$REMOTE_DIR" "$log_file" "$script_file"
      unset POST_PUSH_HOOK || true
    End

    It 'triggers post-push hook with correct arguments and environment'
      REMOTE_DIR="$(mktemp -d)"
      git clone --bare "$TEST_DIR" "$REMOTE_DIR/remote.git" >/dev/null 2>&1
      git remote add origin "$REMOTE_DIR/remote.git"
      git branch -M main

      echo "new line 2" >> README.md
      git add README.md
      git commit -m "feat: second feature"

      log_file="$(mktemp)"
      script_file="$(mktemp)"
      echo '#!/bin/sh' > "$script_file"
      echo 'echo "NATIVE_TRIGGERED: $1 $2" >> "'"$log_file"'"' >> "$script_file"
      echo 'echo "ENV_REMOTE: $CCCP_PUSH_REMOTE" >> "'"$log_file"'"' >> "$script_file"
      chmod +x "$script_file"
      export POST_PUSH_HOOK="$script_file"

      test_run_post_push() {
        run_post_push_hook "origin" "main" "0000000000000000000000000000000000000000" "$(git rev-parse HEAD)"
      }
      When call test_run_post_push
      The status should be success
      The contents of file "$log_file" should include "NATIVE_TRIGGERED: origin main"
      The contents of file "$log_file" should include "ENV_REMOTE: origin"

      rm -rf "$REMOTE_DIR" "$log_file" "$script_file"
      unset POST_PUSH_HOOK || true
    End

    It 'triggers post-push sync actions upon successful push'
      REMOTE_DIR="$(mktemp -d)"
      git clone --bare "$TEST_DIR" "$REMOTE_DIR/remote.git" >/dev/null 2>&1
      git remote add origin "$REMOTE_DIR/remote.git"
      git branch -M main

      echo "sync test" >> README.md
      git add README.md
      git commit -m "feat: third feature"

      cmd_tickets() {
        echo "PUSH_SYNC_TICKETS_TRIGGERED"
      }
      cmd_agent() {
        echo "PUSH_SYNC_AGENTS_$1_TRIGGERED"
      }
      cmd_agents() {
        echo "PUSH_SYNC_AGENTS_$1_TRIGGERED"
      }

      export POST_PUSH_SYNC="tickets, agents"
      test_cmd_push_sync() {
        cmd_push -u origin main 2>/dev/null
      }
      When call test_cmd_push_sync
      The status should be success
      The output should include "PUSH_SYNC_TICKETS_TRIGGERED"
      The output should include "PUSH_SYNC_AGENTS_update_TRIGGERED"

      rm -rf "$REMOTE_DIR"
      unset POST_PUSH_SYNC || true
    End

    It 'fails when executed outside of a git repository'
      NON_GIT_DIR="$(mktemp -d)"
      cd "$NON_GIT_DIR"
      When call cmd_push
      The status should be failure
      The stderr should include "Error: Not a git repository"
      rm -rf "$NON_GIT_DIR"
    End
  End
End

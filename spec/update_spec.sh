# Include the test helper
. "$SHELLSPEC_PROJECT_ROOT/spec/spec_helper.sh"

Describe 'Update and Hook Version Synchronization'
  Include "${CCCP_BUNDLE:-$SHELLSPEC_PROJECT_ROOT/src/utils/config_manager.sh}"
  Include "${CCCP_BUNDLE:-$SHELLSPEC_PROJECT_ROOT/src/utils/update.sh}"

  setup() {
    TEST_DIR="$(mktemp -d)"
    TEST_HOME="$(mktemp -d)"
    cd "$TEST_DIR"

    git init
    git config user.email "test@example.com"
    git config user.name "Test User"
    touch README.md
    git add README.md
    git commit -m "initial commit"

    GIT_ROOT="$TEST_DIR"
    GIT_HOOKS_DIR="$GIT_ROOT/.git/hooks"
    mkdir -p "$GIT_HOOKS_DIR"
    echo "1.5.0" > "$GIT_ROOT/VERSION"
  }

  cleanup() {
    rm -rf "$TEST_DIR" "$TEST_HOME"
  }

  BeforeAll 'setup'
  AfterAll 'cleanup'

  reset_env() {
    cd "$TEST_DIR"
    export HOME="$TEST_HOME"
    export XDG_CONFIG_HOME="$TEST_HOME/.config"
    export CCCP_VERSION="1.5.0"
    rm -rf "$TEST_HOME/.config"
    rm -f "$GIT_HOOKS_DIR/commit-msg" "$GIT_HOOKS_DIR/post-commit"
  }

  Describe 'check_hook_version'
    BeforeEach 'reset_env'

    It 'emits no warning when hooks are matching current version'
      echo "#!/bin/sh" > "$GIT_HOOKS_DIR/commit-msg"
      echo "# cccp-hook-version: 1.5.0" >> "$GIT_HOOKS_DIR/commit-msg"

      When call check_hook_version
      The status should be success
      The stderr should not include "Warning: Git hook"
    End

    It 'warns when hook was installed with an older/different version'
      echo "#!/bin/sh" > "$GIT_HOOKS_DIR/commit-msg"
      echo "# cccp-hook-version: 1.0.0" >> "$GIT_HOOKS_DIR/commit-msg"

      When call check_hook_version
      The status should be success
      The stderr should include "Warning: Git hook 'commit-msg' was installed with cccp v1.0.0 (current: v1.5.0)"
      The stderr should include "Run 'cccp install' to synchronize git hooks"
    End

    It 'ignores non-cccp hooks'
      echo "#!/bin/sh" > "$GIT_HOOKS_DIR/commit-msg"
      echo "echo 'custom non-cccp hook'" >> "$GIT_HOOKS_DIR/commit-msg"

      When call check_hook_version
      The status should be success
      The stderr should not include "Warning: Git hook"
    End
  End

  Describe 'check_auto_update'
    BeforeEach 'reset_env'

    It 'skips update check when CHECK_UPDATES=0'
      export CHECK_UPDATES=0
      When call check_auto_update
      The status should be success
      The stderr should not include "Notice: A newer version"
    End

    It 'notifies user when cached version is newer'
      cache_file="$TEST_HOME/.config/cccp/update_cache"
      mkdir -p "$(dirname "$cache_file")"
      now=$(date +%s)
      cat > "$cache_file" <<EOF
last_check_timestamp=$now
latest_version=2.0.0
EOF

      export CHECK_UPDATES=1
      When call check_auto_update
      The status should be success
      The stderr should include "Notice: A newer version of cccp is available (2.0.0 vs current 1.5.0)"
      The stderr should include "Run 'cccp update' to update"
    End

    It 'does not notify when cached version equals current version'
      cache_file="$TEST_HOME/.config/cccp/update_cache"
      mkdir -p "$(dirname "$cache_file")"
      now=$(date +%s)
      cat > "$cache_file" <<EOF
last_check_timestamp=$now
latest_version=1.5.0
EOF

      export CHECK_UPDATES=1
      When call check_auto_update
      The status should be success
      The stderr should not include "Notice: A newer version"
    End

    It 'suppresses update notice when PINNED_VERSION is set'
      cache_file="$TEST_HOME/.config/cccp/update_cache"
      mkdir -p "$(dirname "$cache_file")"
      now=$(date +%s)
      cat > "$cache_file" <<EOF
last_check_timestamp=$now
latest_version=2.0.0
EOF

      export CHECK_UPDATES=1
      export PINNED_VERSION="1.5.0"
      When call check_auto_update
      The status should be success
      The stderr should not include "Notice: A newer version"
    End
  End

  Describe 'cmd_check_update'
    BeforeEach 'reset_env'

    curl() {
      echo '{"tag_name": "v2.0.0"}'
    }

    It 'displays installed version, release channel, and Gosplan status'
      export CCCP_VERSION="2.0.0"
      export UPDATE_CHANNEL="stable"
      export PINNED_VERSION=""
      When call cmd_check_update
      The status should be success
      The output should include "★ CCCP Update Verification Bureau ★"
      The output should include "Installed Version : 2.0.0"
      The output should include "Release Channel   : stable"
      The output should include "Pinned Version    : none"
    End

    It 'displays pinned status when PINNED_VERSION matches installed version'
      export CCCP_VERSION="2.0.0"
      export PINNED_VERSION="2.0.0"
      When call cmd_check_update
      The status should be success
      The output should include "Pinned Version    : 2.0.0 (Gosplan Directive Active)"
      The output should include "Status            : Pinned to 2.0.0. Updates are frozen by Gosplan decree."
    End
  End

  Describe 'update_script pinning and argument parsing'
    BeforeEach 'reset_env'

    It 'rejects invalid release channels'
      When call update_script --channel invalid_channel
      The status should be failure
      The stderr should include "Invalid update channel 'invalid_channel'"
    End

    It 'pins version to current version when --pin is passed without argument'
      export CCCP_VERSION="2.0.0"
      export PINNED_VERSION=""
      When call update_script --pin
      The status should be success
      The output should include "Gosplan directive enacted: Version pinned to 2.0.0."
      The variable PINNED_VERSION should eq "2.0.0"
    End

    It 'blocks standard update when PINNED_VERSION is set'
      export PINNED_VERSION="2.0.0"
      When call update_script
      The status should be failure
      The stderr should include "Error: Version is pinned to 2.0.0 by Gosplan directive."
      The stderr should include "To upgrade anyway or release the pin, run 'cccp update --unpin'"
    End
  End

  Describe 'get_current_version'
    BeforeEach 'reset_env'

    It 'returns canonical CCCP_VERSION without recursion or external command'
      export CCCP_VERSION="2.0.0"
      When call get_current_version
      The output should eq "2.0.0"
    End
  End
End

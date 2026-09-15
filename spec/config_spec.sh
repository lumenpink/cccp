# Include the test helper
. "$SHELLSPEC_PROJECT_ROOT/spec/spec_helper.sh"

Describe 'Configuration Manager'
  Include "$SHELLSPEC_PROJECT_ROOT/src/utils/config_manager.sh"

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
  }

  cleanup() {
    rm -rf "$TEST_DIR" "$TEST_HOME"
  }

  BeforeAll 'setup'
  AfterAll 'cleanup'

  reset_env() {
    cd "$TEST_DIR"
    export XDG_CONFIG_HOME="$TEST_HOME/.config"
    export HOME="$TEST_HOME"
    rm -f "$TEST_DIR/.cccprc"
    rm -rf "$TEST_HOME/.config"
    unset STRICT_SCOPES STRICT_SUBSCOPES DISABLE_SUBSCOPES DISABLE_MULTIPLE_SCOPES DEFAULT_BASE_VERSION NO_V UPDATE_CHANNEL UPDATE_INTERVAL_DAYS CHECK_UPDATES ALLOW_ANY_SCOPE ALLOW_ANY_SUBSCOPE || true
  }

  Describe 'Low-level key/value operations'
    BeforeEach 'reset_env'

    It 'writes key/value pairs'
      When call write_file_key "$TEST_DIR/.cccprc" "strict_scopes" "1"
      The status should be success
      The file "$TEST_DIR/.cccprc" should be exist
    End

    It 'reads key/value pairs'
      write_file_key "$TEST_DIR/.cccprc" "strict_scopes" "1"
      When call read_file_key "$TEST_DIR/.cccprc" "strict_scopes"
      The status should be success
      The output should equal "1"
    End

    It 'updates existing key without duplicating'
      write_file_key "$TEST_DIR/.cccprc" "strict_scopes" "1"
      write_file_key "$TEST_DIR/.cccprc" "strict_scopes" "0"

      count_lines() { grep -c "strict_scopes" "$TEST_DIR/.cccprc"; }
      When call count_lines
      The output should equal "1"
    End

    It 'reads updated key'
      write_file_key "$TEST_DIR/.cccprc" "strict_scopes" "1"
      write_file_key "$TEST_DIR/.cccprc" "strict_scopes" "0"

      When call read_file_key "$TEST_DIR/.cccprc" "strict_scopes"
      The output should equal "0"
    End

    It 'unsets key cleanly'
      write_file_key "$TEST_DIR/.cccprc" "strict_scopes" "1"
      write_file_key "$TEST_DIR/.cccprc" "no_v" "1"
      unset_file_key "$TEST_DIR/.cccprc" "strict_scopes"

      When call read_file_key "$TEST_DIR/.cccprc" "strict_scopes"
      The status should be failure
    End

    It 'preserves other keys when unsetting'
      write_file_key "$TEST_DIR/.cccprc" "strict_scopes" "1"
      write_file_key "$TEST_DIR/.cccprc" "no_v" "1"
      unset_file_key "$TEST_DIR/.cccprc" "strict_scopes"

      When call read_file_key "$TEST_DIR/.cccprc" "no_v"
      The output should equal "1"
    End
  End

  Describe 'cmd_config CLI interface'
    BeforeEach 'reset_env'

    It 'returns default value when not configured in files'
      When call cmd_config "strict_scopes"
      The status should be success
      The output should equal "0"
    End

    It 'returns default base version'
      When call cmd_config "default_base_version"
      The status should be success
      The output should equal "0.0.1"
    End

    It 'sets local repository configuration'
      When call cmd_config "strict_scopes" "1"
      The status should be success
      The output should include "Set local strict_scopes = 1"
      The file "$TEST_DIR/.cccprc" should be exist
    End

    It 'gets local repository configuration'
      cmd_config "strict_scopes" "1"
      When call cmd_config "strict_scopes"
      The status should be success
      The output should equal "1"
    End

    It 'sets global configuration'
      When call cmd_config --global "update_channel" "nightly"
      The status should be success
      The output should include "Set global update_channel = nightly"
      The file "$TEST_HOME/.config/cccp/config" should be exist
    End

    It 'gets global configuration'
      cmd_config --global "update_channel" "nightly"
      When call cmd_config --global "update_channel"
      The status should be success
      The output should equal "nightly"
    End

    It 'local configuration overrides global configuration'
      cmd_config --global "strict_scopes" "0"
      cmd_config "strict_scopes" "1"

      When call cmd_config "strict_scopes"
      The status should be success
      The output should equal "1"
    End

    It 'allows reading global configuration explicitly'
      cmd_config --global "strict_scopes" "0"
      cmd_config "strict_scopes" "1"

      When call cmd_config --global "strict_scopes"
      The status should be success
      The output should equal "0"
    End

    It 'lists configuration key/value pairs'
      cmd_config --global "update_channel" "stable"
      cmd_config "strict_scopes" "1"

      When call cmd_config --list
      The status should be success
      The output should include "[global] update_channel = stable"
      The output should include "[local] strict_scopes = 1"
    End

    It 'unsets local configuration'
      cmd_config "strict_scopes" "1"
      When call cmd_config --unset "strict_scopes"
      The status should be success
      The output should include "Unset local strict_scopes"
    End

    It 'reverts to default after unsetting key'
      cmd_config "strict_scopes" "1"
      cmd_config --unset "strict_scopes"
      When call cmd_config "strict_scopes"
      The status should be success
      The output should equal "0"
    End
  End

  Describe 'Hierarchical precedence in load_hierarchical_config'
    BeforeEach 'reset_env'

    It 'defaults to internal values when no files exist'
      test_defaults() {
        load_hierarchical_config
        echo "$STRICT_SCOPES"
      }
      When call test_defaults
      The output should equal "0"
    End

    It 'overrides default with global configuration'
      cmd_config --global "strict_scopes" "1" >/dev/null
      test_global() {
        load_hierarchical_config
        echo "$STRICT_SCOPES"
      }
      When call test_global
      The output should equal "1"
    End

    It 'overrides global with local repository configuration'
      cmd_config --global "strict_scopes" "1" >/dev/null
      cmd_config "strict_scopes" "0" >/dev/null
      test_local() {
        load_hierarchical_config
        echo "$STRICT_SCOPES"
      }
      When call test_local
      The output should equal "0"
    End

    It 'overrides local with environment variable'
      cmd_config "strict_scopes" "0" >/dev/null
      test_env() {
        STRICT_SCOPES="1"
        export STRICT_SCOPES
        load_hierarchical_config
        echo "${STRICT_SCOPES}"
      }
      When call test_env
      The output should equal "1"
    End
  End
End

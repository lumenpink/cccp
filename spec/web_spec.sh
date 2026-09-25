# Include the test helper
. "$SHELLSPEC_PROJECT_ROOT/spec/spec_helper.sh"

Describe 'web'
  Include "${CCCP_BUNDLE:-$SHELLSPEC_PROJECT_ROOT/src/utils/web.sh}"
  Include "${CCCP_BUNDLE:-$SHELLSPEC_PROJECT_ROOT/src/utils/comrades.sh}"

  setup() {
    TEST_DIR="$(mktemp -d)"
    cd "$TEST_DIR"
  }

  cleanup() {
    rm -rf "$TEST_DIR"
  }

  BeforeEach 'setup'
  AfterEach 'cleanup'

  Describe '_web_generate_token'
    It 'generates a non-empty hex/alphanumeric token'
      _test_token() {
        tok=$(_web_generate_token)
        [ -n "$tok" ] && [ "${#tok}" -ge 16 ]
      }
      When call _test_token
      The status should be success
    End

    It 'generates distinct tokens across multiple invocations'
      _run_tokens() {
        t1=$(_web_generate_token)
        t2=$(_web_generate_token)
        [ -n "$t1" ] && [ "$t1" != "$t2" ]
      }
      When call _run_tokens
      The status should be success
    End
  End

  Describe '_web_export_assets'
    It 'creates target directory, writes self-contained index.html without token leak, and includes auth modal'
      target_dir="$TEST_DIR/web_assets"
      token="gosplan-token-1922"
      When call _web_export_assets "$target_dir" "$token" 8080
      The status should be success
      The path "$target_dir/index.html" should be exist
      The file "$target_dir/index.html" should be file
      The contents of file "$target_dir/index.html" should include "CCCP"
      The contents of file "$target_dir/index.html" should include "auth-modal"
      The contents of file "$target_dir/index.html" should include "repo-select"
      The contents of file "$target_dir/index.html" should not include "gosplan-token-1922"
      The contents of file "$target_dir/index.html" should include "8080"
    End
  End

  Describe 'cmd_web'
    It 'displays usage help on -h or --help'
      When call cmd_web --help
      The status should be success
      The output should include "cccp web"
      The output should include "--port"
      The output should include "--host"
      The output should include "--open"
      The output should include "--token"
    End

    It 'launches mk with bridge and auth parameters without opening browser by default'
      # Mock mk in PATH
      mkdir -p "$TEST_DIR/bin"
      cat > "$TEST_DIR/bin/mk" << 'EOF'
#!/bin/sh
echo "MOCK_MK_ARGS: $@"
exit 0
EOF
      chmod +x "$TEST_DIR/bin/mk"
      PATH="$TEST_DIR/bin:$PATH"

      When call cmd_web --port 8088 --host 127.0.0.1
      The status should be success
      The output should include "MOCK_MK_ARGS:"
      The output should include "--bridge"
      The output should include "--auth"
      The output should include "-p 8088"
      The output should include "--host 127.0.0.1"
      The output should not include "--open"
    End

    It 'passes --open to mk when explicitly requested'
      mkdir -p "$TEST_DIR/bin"
      cat > "$TEST_DIR/bin/mk" << 'EOF'
#!/bin/sh
echo "MOCK_MK_ARGS: $@"
exit 0
EOF
      chmod +x "$TEST_DIR/bin/mk"
      PATH="$TEST_DIR/bin:$PATH"

      When call cmd_web --open --port 8088 --host 127.0.0.1
      The status should be success
      The output should include "MOCK_MK_ARGS:"
      The output should include "--open"
    End

    It 'uses token provided via --token flag'
      mkdir -p "$TEST_DIR/bin"
      cat > "$TEST_DIR/bin/mk" << 'EOF'
#!/bin/sh
echo "MOCK_MK_ARGS: $@"
exit 0
EOF
      chmod +x "$TEST_DIR/bin/mk"
      PATH="$TEST_DIR/bin:$PATH"

      When call cmd_web --token "super-secret-key-123" --port 8088
      The status should be success
      The output should include "--auth super-secret-key-123"
    End

    It 'uses token provided via CCCP_TOKEN environment variable'
      mkdir -p "$TEST_DIR/bin"
      cat > "$TEST_DIR/bin/mk" << 'EOF'
#!/bin/sh
echo "MOCK_MK_ARGS: $@"
exit 0
EOF
      chmod +x "$TEST_DIR/bin/mk"
      PATH="$TEST_DIR/bin:$PATH"

      CCCP_TOKEN="env-secret-token-456"
      export CCCP_TOKEN
      When call cmd_web --port 8088
      The status should be success
      The output should include "--auth env-secret-token-456"
      unset CCCP_TOKEN
    End
  End

  Describe 'global repository context (-C)'
    It 'switches directory and executes in target repository'
      mkdir -p "$TEST_DIR/sample_repo"
      git -C "$TEST_DIR/sample_repo" init -q
      printf '3.2.1\n' > "$TEST_DIR/sample_repo/VERSION"

      When call "$SHELLSPEC_PROJECT_ROOT/cccp.sh" -C "$TEST_DIR/sample_repo" status
      The status should be success
      The output should include "Repository:"
      The output should include "$TEST_DIR/sample_repo"
      The output should include "Current Version:  3.2.1"
    End

    It 'fails when target directory does not exist'
      When call "$SHELLSPEC_PROJECT_ROOT/cccp.sh" -C "$TEST_DIR/non_existent_folder_xyz" status
      The status should be failure
      The error should include "Error: Repository directory"
    End
  End

  Describe 'strict zero-awk compliance'
    It 'verifies zero executable awk invocations in src/utils/web.sh'
      _check_zero_awk() {
        file="$SHELLSPEC_PROJECT_ROOT/src/utils/web.sh"
        if [ -f "$file" ]; then
          awk_calls=$(grep -E '^[[:space:]]*awk[[:space:]]|(\||\&\&|;)[[:space:]]*awk[[:space:]]' "$file" 2>/dev/null || true)
          [ -z "$awk_calls" ]
        fi
      }
      When call _check_zero_awk
      The status should be success
    End
  End
End

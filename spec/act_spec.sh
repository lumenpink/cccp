# Include the test helper
. "$SHELLSPEC_PROJECT_ROOT/spec/spec_helper.sh"

Describe 'Act Local CI Runner & Pre-Push Gate'
  Include "${CCCP_BUNDLE:-$SHELLSPEC_PROJECT_ROOT/src/utils/act.sh}"
  Include "${CCCP_BUNDLE:-$SHELLSPEC_PROJECT_ROOT/src/hooks/hooks.sh}"
  Include "${CCCP_BUNDLE:-$SHELLSPEC_PROJECT_ROOT/src/utils/help.sh}"

  setup() {
    TEST_WORKSPACE="$(mktemp -d)"
    export NO_COLOR="1"
    export CCCP_ACT_SKIP=""
    export ACT_SKIP=""

    # Initialize a test git repo
    (
      cd "$TEST_WORKSPACE"
      git init -q
      git config user.name "Soviet Tester"
      git config user.email "tester@gosplan.su"
      echo "hello" > file.txt
      git add file.txt
      git commit -q -m "feat: initial commit"
    )
  }

  cleanup() {
    rm -rf "$TEST_WORKSPACE"
    unset NO_COLOR CCCP_ACT_SKIP ACT_SKIP CCCP_ACT_BIN CCCP_ACT_JOB
  }

  BeforeEach 'setup'
  AfterEach 'cleanup'

  Describe '_act_format_time'
    It 'formats 0 seconds as 0m 00s'
      When call _act_format_time 0
      The status should be success
      The output should eq "0m 00s"
    End

    It 'formats 47 seconds as 0m 47s'
      When call _act_format_time 47
      The status should be success
      The output should eq "0m 47s"
    End

    It 'formats 94 seconds as 1m 34s'
      When call _act_format_time 94
      The status should be success
      The output should eq "1m 34s"
    End
  End

  Describe '_act_check_prerequisites'
    It 'returns 1 when act binary does not exist'
      export CCCP_ACT_BIN="/nonexistent/act_binary_xyz"
      When call _act_check_prerequisites
      The status should eq 1
    End
  End

  Describe '_act_find_workflow'
    It 'returns failure when no workflow exists'
      When call _act_find_workflow "$TEST_WORKSPACE"
      The status should be failure
    End

    It 'finds .forgejo/workflows/ci.yml'
      mkdir -p "$TEST_WORKSPACE/.forgejo/workflows"
      touch "$TEST_WORKSPACE/.forgejo/workflows/ci.yml"
      When call _act_find_workflow "$TEST_WORKSPACE"
      The status should be success
      The output should eq "$TEST_WORKSPACE/.forgejo/workflows/ci.yml"
    End

    It 'finds .github/workflows/ci.yml'
      mkdir -p "$TEST_WORKSPACE/.github/workflows"
      touch "$TEST_WORKSPACE/.github/workflows/ci.yml"
      When call _act_find_workflow "$TEST_WORKSPACE"
      The status should be success
      The output should eq "$TEST_WORKSPACE/.github/workflows/ci.yml"
    End
  End

  Describe 'act_ci_gate'
    It 'bypasses gate when CCCP_ACT_SKIP is set'
      _run_gate() {
        cd "$TEST_WORKSPACE"
        CCCP_ACT_SKIP=1 act_ci_gate "origin"
      }
      When call _run_gate
      The status should be success
      The output should include "bypassed"
    End

    It 'skips gate when no workflow is defined in repo'
      _run_gate() {
        cd "$TEST_WORKSPACE"
        act_ci_gate "origin"
      }
      When call _run_gate
      The status should be success
      The output should eq ""
    End

    It 'degrades gracefully and does not block push when act is unavailable'
      _run_gate() {
        cd "$TEST_WORKSPACE"
        mkdir -p .forgejo/workflows
        touch .forgejo/workflows/ci.yml
        CCCP_ACT_BIN="/nonexistent/act" act_ci_gate "origin"
      }
      When call _run_gate
      The status should be success
      The output should include "degradação graciosa"
    End

    It 'allows push and creates structured log when act succeeds'
      _run_success_gate() {
        cd "$TEST_WORKSPACE"
        mkdir -p .forgejo/workflows
        touch .forgejo/workflows/ci.yml

        # Mock successful act runner
        _mock_act="$TEST_WORKSPACE/mock_act.sh"
        cat > "$_mock_act" << 'MOCK'
#!/bin/sh
echo "[test/Lint] ⭐ Run composer test"
echo "All tests passed successfully!"
exit 0
MOCK
        chmod +x "$_mock_act"

        # Mock docker check to pass
        docker() { return 0; }

        CCCP_ACT_BIN="$_mock_act" act_ci_gate "origin"
      }
      When call _run_success_gate
      The status should be success
      The output should include "CI gate passed"
      The output should include "pushing to origin"

      # Verify log file was created in .git/act-logs/
      _log_files=$(ls "$TEST_WORKSPACE/.git/act-logs/"*.log 2>/dev/null | wc -l | tr -d ' ')
      [ "$_log_files" -ge 1 ] || return 1
    End

    It 'aborts push with error summary and log path when act fails'
      _run_failure_gate() {
        cd "$TEST_WORKSPACE"
        mkdir -p .forgejo/workflows
        touch .forgejo/workflows/ci.yml

        # Mock failing act runner
        _mock_act="$TEST_WORKSPACE/mock_act_fail.sh"
        cat > "$_mock_act" << 'MOCK'
#!/bin/sh
echo "[test/Unit Tests] ⭐ Run cargo test"
echo "##[error] cargo test failed: 2 assertion errors"
exit 1
MOCK
        chmod +x "$_mock_act"

        # Mock docker check to pass
        docker() { return 0; }

        CCCP_ACT_BIN="$_mock_act" act_ci_gate "origin"
      }
      When call _run_failure_gate
      The status should be failure
      The output should include "CI gate failed"
      The output should include "Step failed:"
      The output should include "cargo test failed"
      The output should include "Log: "
      The output should include "git push --no-verify"
    End
  End

  Describe 'cmd_act'
    It 'reports error when trying to view log but no logs exist'
      _view_log() {
        cd "$TEST_WORKSPACE"
        cmd_act log
      }
      When call _view_log
      The status should be failure
      The error should include "No act CI logs found"
    End

    It 'displays the most recent log file'
      _view_log() {
        cd "$TEST_WORKSPACE"
        mkdir -p .git/act-logs
        echo "LOG CONTENT TEST 1234" > .git/act-logs/2026-09-26_08-00_main_abc.log
        cmd_act log
      }
      When call _view_log
      The status should be success
      The output should include "LOG CONTENT TEST 1234"
    End

    It 'displays runner status'
      _show_status() {
        cd "$TEST_WORKSPACE"
        cmd_act status
      }
      When call _show_status
      The status should be success
      The output should include "CCCP Act Local CI Runner Status"
      The output should include "Docker Daemon:"
    End

    It 'displays help message'
      When call cmd_act -h
      The status should be success
      The output should include "cccp.sh act - Local CI Runner & Gate"
    End
  End
End

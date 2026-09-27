# Include the test helper
. "$SHELLSPEC_PROJECT_ROOT/spec/spec_helper.sh"

Describe 'matraca'
  Include "${CCCP_BUNDLE:-$SHELLSPEC_PROJECT_ROOT/src/utils/matraca.sh}"

  setup() {
    TEST_DATA_DIR="$(mktemp -d)"
    export MATRACA_DATA_DIR="$TEST_DATA_DIR"
    export MATRACA_NO_AUTO_WORKER=1
    unset MATRACA_RUNNER || true
  }

  cleanup() {
    rm -rf "$TEST_DATA_DIR"
  }

  BeforeEach 'setup'
  AfterEach 'cleanup'

  Describe '_matraca_escape_json'
    It 'escapes quotes, backslashes, tabs, and newlines in pure POSIX'
      When call _matraca_escape_json 'linha "1"\teste\outro'
      The status should be success
      The output should eq 'linha \"1\"\\teste\\outro'
    End
  End

  Describe 'cmd_matraca_send and cmd_matraca_poll'
    It 'rejects empty prompt'
      When call cmd_matraca_send
      The status should be failure
      The error should include 'Prompt cannot be empty'
    End

    It 'enqueues prompt, creates pending file, and returns status pending'
      When call cmd_matraca_send --client-id "cli-test-100" "List all branches"
      The status should be success
      The output should include '"ok": true'
      The output should include '"client_id": "cli-test-100"'
      The output should include '"status": "pending"'
    End

    It 'preserves idempotency on duplicate client_id'
      cmd_matraca_send --client-id "cli-dup-1" "Original prompt" >/dev/null
      When call cmd_matraca_send --client-id "cli-dup-1" "Retry prompt"
      The status should be success
      The output should include '"prompt": "Original prompt"'
      The output should include '"client_id": "cli-dup-1"'
    End

    It 'polls existing pending job by client_id'
      cmd_matraca_send --client-id "cli-poll-1" "Test polling" >/dev/null
      When call cmd_matraca_poll "cli-poll-1"
      The status should be success
      The output should include '"status": "pending"'
      The output should include '"client_id": "cli-poll-1"'
    End

    It 'returns error when polling nonexistent job'
      When call cmd_matraca_poll "non-existent-job-xyz"
      The status should be failure
      The error should include 'not_found'
    End
  End

  Describe 'cmd_matraca_worker'
    It 'processes pending job successfully with mock runner'
      export MATRACA_RUNNER="echo 'Mocked agent response 42'"
      cmd_matraca_send --client-id "cli-work-1" "Do something" >/dev/null
      cmd_matraca_worker --once
      When call cmd_matraca_poll "cli-work-1"
      The status should be success
      The output should include '"status": "done"'
      The output should include 'Mocked agent response 42'
      The output should include '"exit_code": 0'
    End

    It 'records error status when runner exits with failure'
      export MATRACA_RUNNER="echo 'Falha simulada no agente' >&2; exit 2"
      cmd_matraca_send --client-id "cli-err-1" "Trigger error" >/dev/null
      cmd_matraca_worker --once
      When call cmd_matraca_poll "cli-err-1"
      The status should be success
      The output should include '"status": "error"'
      The output should include 'Falha simulada no agente'
      The output should include '"exit_code": 2'
    End
  End

  Describe 'cmd_matraca_status and cmd_matraca_history'
    It 'returns structured json metrics in cmd_matraca_status'
      export MATRACA_RUNNER="echo 'ok'"
      cmd_matraca_send --client-id "c1" "p1" >/dev/null
      cmd_matraca_send --client-id "c2" "p2" >/dev/null
      cmd_matraca_worker --once >/dev/null
      When call cmd_matraca_status --json
      The status should be success
      The output should include '"ok": true'
      The output should include '"done": 2'
      The output should include '"pending": 0'
      The output should include '"clients": 2'
    End

    It 'returns array of messages in cmd_matraca_history'
      export MATRACA_RUNNER="echo 'result-hist'"
      cmd_matraca_send --client-id "c-hist" "history test prompt" >/dev/null
      cmd_matraca_worker --once >/dev/null
      When call cmd_matraca_history --limit 10
      The status should be success
      The output should include '"ok": true'
      The output should include '"messages": ['
      The output should include 'history test prompt'
      The output should include 'result-hist'
    End
  End

  Describe 'cmd_matraca web dispatch'
    It 'dispatches to cmd_cockpit on web subcommand'
      cmd_cockpit() {
        echo "MOCK_CMD_COCKPIT_CALLED: $@"
        return 0
      }
      When call cmd_matraca web --port 9090
      The status should be success
      The output should include "MOCK_CMD_COCKPIT_CALLED: matraca --port 9090"
      The error should include "cccp cockpit"
    End
  End

  Describe 'strict zero-awk compliance'
    It 'verifies zero executable awk invocations in src/utils/matraca.sh'
      test_awk() {
        grep -rnw "awk" "$SHELLSPEC_PROJECT_ROOT/src/utils/matraca.sh" | grep -v '#.*awk' || true
      }
      When call test_awk
      The status should be success
      The output should eq ""
    End
  End
End

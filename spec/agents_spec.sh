# Include the test helper
. "$SHELLSPEC_PROJECT_ROOT/spec/spec_helper.sh"

Describe 'agents'
  Include "${CCCP_BUNDLE:-$SHELLSPEC_PROJECT_ROOT/src/utils/help.sh}"
  Include "${CCCP_BUNDLE:-$SHELLSPEC_PROJECT_ROOT/src/utils/config_manager.sh}"
  Include "${CCCP_BUNDLE:-$SHELLSPEC_PROJECT_ROOT/src/utils/agents.sh}"

  setup() {
    TEST_DIR="$(mktemp -d)"
    cd "$TEST_DIR"

    git init -q
    git config user.email "test@example.com"
    git config user.name "Test User"
    echo "1.0.0" > "$TEST_DIR/VERSION"
    git add VERSION
    git commit -q -m "chore: initial commit"

    GIT_ROOT="$TEST_DIR"
    unset AGENTS_MD_URL || true
    unset AGENTS_MD_FILE || true
    unset AGENT_VAR_PROJECT_NAME || true
    unset AGENT_VAR_STACK || true
  }

  cleanup() {
    rm -rf "$TEST_DIR"
  }

  BeforeEach 'setup'
  AfterEach 'cleanup'

  Describe 'extract_template_vars'
    It 'extracts both mustache and shell style variables'
      template="Hello {{PROJECT_NAME}}, using \${TECH_STACK} and {{TEST_CMD}}!"
      When call extract_template_vars "$template"
      The status should be success
      The line 1 of output should eq "PROJECT_NAME"
      The line 2 of output should eq "TECH_STACK"
      The line 3 of output should eq "TEST_CMD"
    End

    It 'returns empty when no variables exist'
      When call extract_template_vars "Static text with no variables."
      The status should be success
      The output should eq ""
    End
  End

  Describe 'cmd_agents set, get and unset'
    It 'sets variable in local .cccprc'
      When call cmd_agents set project_name "my-app"
      The status should be success
      The output should include "Configured agent variable 'project_name' = 'my-app'"
      The path "$TEST_DIR/.cccprc" should be exist
    End

    It 'strips agent_var_ prefix if user provided it'
      When call cmd_agents set agent_var_stack "Rust"
      The status should be success
      The output should include "Configured agent variable 'stack' = 'Rust'"
    End

    It 'works via cmd_set_av shortcut'
      When call cmd_set_av test_cmd "cargo test"
      The status should be success
      The output should include "Configured agent variable 'test_cmd' = 'cargo test'"
    End

    It 'gets existing variable'
      cmd_agents set project_name "rocket" >/dev/null 2>&1
      When call cmd_agents get project_name
      The status should be success
      The output should eq "rocket"
    End

    It 'fails when getting unset variable'
      When call cmd_agents get non_existent_var
      The status should be failure
      The error should include "Error: Variable 'non_existent_var' is not set."
    End

    It 'unsets variable from .cccprc'
      cmd_agents set project_name "rocket" >/dev/null 2>&1
      When call cmd_agents unset project_name
      The status should be success
      The output should include "Unset agent variable 'project_name'"
    End
  End

  Describe 'cmd_agents check'
    It 'detects missing variables and returns 1'
      tmpl_file="$TEST_DIR/template.md"
      printf 'Name: {{PROJECT_NAME}}\nStack: ${TECH_STACK}\n' > "$tmpl_file"
      cmd_agents set project_name "awesome" >/dev/null 2>&1

      When call cmd_agents check --url "$tmpl_file"
      The status should be failure
      The output should include "[✓] PROJECT_NAME = awesome"
      The output should include "[✗] TECH_STACK = (missing"
      The output should include "Status: 1 variable(s) missing"
    End

    It 'reports all satisfied when all variables are configured'
      tmpl_file="$TEST_DIR/template.md"
      printf 'Name: {{PROJECT_NAME}}\n' > "$tmpl_file"
      cmd_agents set project_name "awesome" >/dev/null 2>&1

      When call cmd_agents check --url "$tmpl_file"
      The status should be success
      The output should include "[✓] PROJECT_NAME = awesome"
      The output should include "Status: All variables configured!"
    End
  End

  Describe 'cmd_agents update / generate'
    It 'aborts and returns 1 when variables are missing without creating file'
      tmpl_file="$TEST_DIR/template.md"
      out_file="$TEST_DIR/AGENTS.md"
      printf 'Name: {{PROJECT_NAME}}\nStack: ${TECH_STACK}\n' > "$tmpl_file"

      test_abort_missing() {
        cmd_agents --url "$tmpl_file" --file "$out_file"
      }

      When call test_abort_missing
      The status should be failure
      The output should include "Fetching AGENTS template from:"
      The error should include "Error: Missing required variables for"
      The error should include "PROJECT_NAME"
      The error should include "TECH_STACK"
      The error should include "Aborting generation of"
      The path "$out_file" should not be exist
    End

    It 'interpolates both {{VAR}} and ${VAR} and generates target file'
      tmpl_file="$TEST_DIR/template.md"
      out_file="$TEST_DIR/AGENTS.md"
      printf 'Project: {{PROJECT_NAME}}\nStack: ${TECH_STACK}\n' > "$tmpl_file"

      cmd_agents set project_name "SuperCCCP" >/dev/null 2>&1
      cmd_agents set tech_stack "POSIX Shell" >/dev/null 2>&1

      test_generate_success() {
        cmd_agents --url "$tmpl_file" --file "$out_file"
      }

      When call test_generate_success
      The status should be success
      The output should include "Done! Generated"
      The path "$out_file" should be exist
      The file "$out_file" should be file
      The contents of file "$out_file" should include "Project: SuperCCCP"
      The contents of file "$out_file" should include "Stack: POSIX Shell"
    End
  End
End

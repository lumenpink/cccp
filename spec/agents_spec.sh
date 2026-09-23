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

  Describe 'cmd_agent set, get and unset'
    It 'sets variable in local .cccprc'
      When call cmd_agent set project_name "my-app"
      The status should be success
      The output should include "Configured agent variable 'project_name' = 'my-app'"
      The path "$TEST_DIR/.cccprc" should be exist
    End

    It 'configures agent_md_url directly when setting url'
      When call cmd_agent set url "https://example.com/custom.md"
      The status should be success
      The output should include "Configured agent template URL 'agent_md_url'"
      The contents of file "$TEST_DIR/.cccprc" should include "agent_md_url = https://example.com/custom.md"
      The contents of file "$TEST_DIR/.cccprc" should not include "agent_var_url"
    End

    It 'configures agent_md_file directly when setting file'
      When call cmd_agent set file ".antigravity/CUSTOM.md"
      The status should be success
      The output should include "Configured agent destination file 'agent_md_file'"
      The contents of file "$TEST_DIR/.cccprc" should include "agent_md_file = .antigravity/CUSTOM.md"
    End

    It 'strips agent_var_ prefix if user provided it'
      When call cmd_agent set agent_var_stack "Rust"
      The status should be success
      The output should include "Configured agent variable 'stack' = 'Rust'"
    End

    It 'works via cmd_set_av shortcut'
      When call cmd_set_av test_cmd "cargo test"
      The status should be success
      The output should include "Configured agent variable 'test_cmd' = 'cargo test'"
    End

    It 'gets existing variable'
      cmd_agent set project_name "rocket" >/dev/null 2>&1
      When call cmd_agent get project_name
      The status should be success
      The output should eq "rocket"
    End

    It 'fails when getting unset variable'
      When call cmd_agent get non_existent_var
      The status should be failure
      The error should include "Error: Variable 'non_existent_var' is not set."
    End

    It 'unsets variable from .cccprc'
      cmd_agent set project_name "rocket" >/dev/null 2>&1
      When call cmd_agent unset project_name
      The status should be success
      The output should include "Unset agent variable 'project_name'"
    End
  End

  Describe 'flexible variable resolution'
    It 'resolves AGENTS_LANG when agent_var_lang is configured'
      cmd_agent set lang "english" >/dev/null 2>&1
      When call resolve_agent_var "AGENTS_LANG"
      The status should be success
      The output should eq "english"
    End

    It 'resolves LANG when agent_var_agents_lang is configured'
      echo "agent_var_agents_lang = english" >> "$TEST_DIR/.cccprc"
      When call resolve_agent_var "LANG"
      The status should be success
      The output should eq "english"
    End
  End

  Describe 'cmd_agent check'
    It 'detects missing variables and returns 1'
      tmpl_file="$TEST_DIR/template.md"
      printf 'Name: {{PROJECT_NAME}}\nStack: ${TECH_STACK}\n' > "$tmpl_file"
      cmd_agent set project_name "awesome" >/dev/null 2>&1

      When call cmd_agent check --url "$tmpl_file"
      The status should be failure
      The output should include "[✓] PROJECT_NAME = awesome"
      The output should include "[✗] TECH_STACK = (missing"
      The output should include "Status: 1 variable(s) missing"
    End

    It 'reports all satisfied when all variables are configured'
      tmpl_file="$TEST_DIR/template.md"
      printf 'Name: {{PROJECT_NAME}}\n' > "$tmpl_file"
      cmd_agent set project_name "awesome" >/dev/null 2>&1

      When call cmd_agent check --url "$tmpl_file"
      The status should be success
      The output should include "[✓] PROJECT_NAME = awesome"
      The output should include "Status: All variables configured!"
    End

    It 'uses configured agent_md_url from .cccprc when no --url is specified'
      tmpl_file="$TEST_DIR/custom_tmpl.md"
      printf 'Greeting: {{GREETING}}\n' > "$tmpl_file"
      echo "agent_md_url = $tmpl_file" >> "$TEST_DIR/.cccprc"
      cmd_agent set greeting "Hello world" >/dev/null 2>&1

      When call cmd_agent check
      The status should be success
      The output should include "Template: $tmpl_file"
      The output should include "[✓] GREETING = Hello world"
      The output should include "Status: All variables configured!"
    End
  End

  Describe 'cmd_agent update / generate'
    It 'generates file and warns when variables are missing'
      tmpl_file="$TEST_DIR/template.md"
      out_file="$TEST_DIR/.antigravity/AGENTS.md"
      printf 'Name: {{PROJECT_NAME}}\nStack: ${TECH_STACK}\n' > "$tmpl_file"

      test_warn_missing() {
        cmd_agent --url "$tmpl_file" --file "$out_file"
      }

      When call test_warn_missing
      The status should be success
      The output should include "Fetching AGENTS template from:"
      The output should include "Done! Generated"
      The error should include "Warning: 2 variable(s) missing in"
      The error should include "project_name"
      The error should include "tech_stack"
      The error should include "Run 'cccp agent fill' to interactively provide missing variables."
      The path "$out_file" should be exist
      The contents of file "$out_file" should include "Name: {{PROJECT_NAME}}"
      The contents of file "$out_file" should include 'Stack: ${TECH_STACK}'
    End

    It 'interpolates variables, generates default .antigravity/AGENTS.md, and adds to .gitignore'
      tmpl_file="$TEST_DIR/template.md"
      printf 'Project: {{PROJECT_NAME}}\nLang: ${AGENTS_LANG}\n' > "$tmpl_file"

      cmd_agent set project_name "SuperCCCP" >/dev/null 2>&1
      cmd_agent set lang "portuguese" >/dev/null 2>&1

      test_generate_default() {
        cmd_agent --url "$tmpl_file"
      }

      When call test_generate_default
      The status should be success
      The output should include "Done! Generated"
      The path "$TEST_DIR/.antigravity/AGENTS.md" should be exist
      The file "$TEST_DIR/.antigravity/AGENTS.md" should be file
      The contents of file "$TEST_DIR/.antigravity/AGENTS.md" should include "Project: SuperCCCP"
      The contents of file "$TEST_DIR/.antigravity/AGENTS.md" should include "Lang: portuguese"
      The contents of file "$TEST_DIR/.gitignore" should include ".antigravity/"
    End

    It 'safely interpolates variables containing sed-sensitive and special characters'
      tmpl_file="$TEST_DIR/template.md"
      out_file="$TEST_DIR/.antigravity/AGENTS.md"
      printf 'URL: {{API_URL}}\nNote: ${SPECIAL_NOTE}\nRepeat: {{API_URL}}\n' > "$tmpl_file"

      cmd_agent set api_url "https://api.example.com/v1?a=1&b=2/c\\d" >/dev/null 2>&1
      cmd_agent set special_note "Special & chars / and \"double\" quotes" >/dev/null 2>&1

      When call cmd_agent --url "$tmpl_file" --file "$out_file"
      The status should be success
      The output should include "Done! Generated"
      The contents of file "$out_file" should include 'URL: https://api.example.com/v1?a=1&b=2/c\d'
      The contents of file "$out_file" should include 'Note: Special & chars / and "double" quotes'
      The contents of file "$out_file" should include 'Repeat: https://api.example.com/v1?a=1&b=2/c\d'
    End
  End

  Describe 'cmd_agent fill'
    It 'prompts interactively for missing variables and regenerates file'
      tmpl_file="$TEST_DIR/template.md"
      out_file="$TEST_DIR/.antigravity/AGENTS.md"
      printf 'Name: {{PROJECT_NAME}}\nStack: ${TECH_STACK}\n' > "$tmpl_file"
      cmd_agent set project_name "MyProject" >/dev/null 2>&1

      # Pipe input into cmd_agent fill
      test_fill() {
        printf 'posix-shell\n' | cmd_agent fill --url "$tmpl_file" --file "$out_file"
      }

      When call test_fill
      The status should be success
      The output should include "[✓] project_name = MyProject"
      The output should include "Saved 1 variable(s). Regenerating"
      The path "$out_file" should be exist
      The contents of file "$out_file" should include "Name: MyProject"
      The contents of file "$out_file" should include "Stack: posix-shell"
      The contents of file "$TEST_DIR/.cccprc" should include "agent_var_tech_stack = posix-shell"
    End
  End

  Describe 'ensure_agent_gitignored'
    It 'adds .antigravity/ directory to .gitignore'
      rm -f "$TEST_DIR/.gitignore"
      When call ensure_agent_gitignored "$TEST_DIR/.antigravity/AGENTS.md"
      The status should be success
      The file "$TEST_DIR/.gitignore" should be exist
      The contents of file "$TEST_DIR/.gitignore" should include ".antigravity/"
    End

    It 'is idempotent and does not duplicate entries'
      echo ".antigravity/" > "$TEST_DIR/.gitignore"
      ensure_and_count() {
        ensure_agent_gitignored "$TEST_DIR/.antigravity/AGENTS.md"
        grep -c "^\.antigravity/" "$TEST_DIR/.gitignore"
      }
      When call ensure_and_count
      The status should be success
      The output should eq "1"
    End
  End

  Describe 'fetch_agent_template'
    It 'fetches template from local file path'
      local_tmpl="$TEST_DIR/my_local_template.md"
      echo "# Direct Local Template" > "$local_tmpl"
      When call fetch_agent_template "$local_tmpl"
      The status should be success
      The output should include "# Direct Local Template"
    End

    It 'fails when local template file does not exist'
      When call fetch_agent_template "$TEST_DIR/nonexistent_file.md"
      The status should be failure
    End
  End

  Describe '_posix_replace_all'
    It 'replaces multiple occurrences in string'
      When call _posix_replace_all "foo bar foo baz foo" "foo" "qux"
      The status should be success
      The output should eq "qux bar qux baz qux"
    End

    It 'handles special characters like &, /, backslashes and quotes without corruption'
      When call _posix_replace_all "start {{TAG}} end" "{{TAG}}" 'val & / \ " end'
      The status should be success
      The output should eq 'start val & / \ " end end'
    End

    It 'returns original string when needle is empty or not found'
      When call _posix_replace_all "unchanged text" "" "anything"
      The status should be success
      The output should eq "unchanged text"
    End
  End

  Describe 'strict zero-awk compliance'
    It 'verifies zero executable awk invocations across src/ modules'
      test_awk() {
        grep -rnw "awk" "$SHELLSPEC_PROJECT_ROOT/src" | grep -v '#.*awk' || true
      }
      When call test_awk
      The status should be success
      The output should eq ""
    End
  End
End

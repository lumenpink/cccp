# Include the test helper
. "$SHELLSPEC_PROJECT_ROOT/spec/spec_helper.sh"

Describe 'diva'
  Include "${CCCP_BUNDLE:-$SHELLSPEC_PROJECT_ROOT/src/utils/help.sh}"
  Include "${CCCP_BUNDLE:-$SHELLSPEC_PROJECT_ROOT/src/utils/config_manager.sh}"
  Include "${CCCP_BUNDLE:-$SHELLSPEC_PROJECT_ROOT/src/utils/repo.sh}"
  Include "${CCCP_BUNDLE:-$SHELLSPEC_PROJECT_ROOT/src/utils/diva.sh}"

  setup() {
    TEST_DIR="$(mktemp -d)"
    cd "$TEST_DIR"
    git init -q
    git config user.email "test@example.com"
    git config user.name "Test User"
    git config remote.origin.url "https://codeberg.org/lumenpink/myproject.git"

    export HOME="$TEST_DIR"
    unset GEMINI_API_KEY || true
    unset CODEBERG_TOKEN || true
    unset TILDEGIT_TOKEN || true
    unset GITHUB_TOKEN || true
  }

  cleanup() {
    rm -rf "$TEST_DIR"
  }

  BeforeEach 'setup'
  AfterEach 'cleanup'

  Describe '_diva_resolve_api_key'
    It 'resolves key from GEMINI_API_KEY environment variable'
      export GEMINI_API_KEY="AIzaSyTestEnvKey123"
      When call _diva_resolve_api_key
      The status should be success
      The output should eq "AIzaSyTestEnvKey123"
    End

    It 'resolves key from ~/.env when unset in memory'
      unset GEMINI_API_KEY || true
      echo "GEMINI_API_KEY=AIzaSyDotEnvKey789" > "$TEST_DIR/.env"
      When call _diva_resolve_api_key
      The status should be success
      The output should eq "AIzaSyDotEnvKey789"
    End

    It 'resolves key from .cccprc config'
      echo "gemini_api_key = AIzaSyConfigFileKey456" > "$TEST_DIR/.cccprc"
      When call _diva_resolve_api_key
      The status should be success
      The output should eq "AIzaSyConfigFileKey456"
    End

    It 'fails when no key is configured'
      When call _diva_resolve_api_key
      The status should be failure
    End
  End

  Describe '_diva_escape_json'
    It 'escapes quotes, backslashes, and converts newlines to \\n'
      sample="Line 1: with \"quotes\" and \\backslash
Line 2: second line"
      When call _diva_escape_json "$sample"
      The status should be success
      The output should eq 'Line 1: with \"quotes\" and \\backslash\nLine 2: second line'
    End
  End

  Describe '_diva_extract_tickets'
    It 'extracts tickets JSON payload from raw Gemini API candidate response'
      raw_resp='{
  "candidates": [
    {
      "content": {
        "parts": [
          {
            "text": "{\n  \"tickets\": [\n    {\n      \"title\": \"fix(auth): handle token expiration\",\n      \"rationale\": \"Separate token refresh bug\",\n      \"labels\": [\"bug\", \"auth\"],\n      \"body\": \"## Description\\nSession timeout fails.\"\n    }\n  ]\n}"
          }
        ]
      }
    }
  ]
}'
      When call _diva_extract_tickets "$raw_resp"
      The status should be success
      The output should include '"title": "fix(auth): handle token expiration"'
      The output should include '"rationale": "Separate token refresh bug"'
    End

    It 'fails when raw response has no valid text part'
      When call _diva_extract_tickets '{"candidates": []}'
      The status should be failure
    End
  End

  Describe '_diva_present_and_dispatch'
    It 'previews ticket box in dry-run mode without publishing'
      sample_tickets='{
  "tickets": [
    {
      "title": "feat(ui): add dark mode toggle",
      "rationale": "Visual preference separation",
      "labels": ["ui", "enhancement"],
      "body": "## Description\nUser toggle for dark theme in header."
    }
  ]
}'
      When call _diva_present_and_dispatch "$sample_tickets" "codeberg" "lumenpink" "myproject" 1
      The status should be success
      The output should include "O Divã identificou 1 ticket(s)"
      The output should include "[ДИВАН] Ticket 1/1"
      The output should include "Destino  : lumenpink/myproject (codeberg)"
      The output should include "feat(ui): add dark mode toggle"
      The output should include "Visual preference separation"
      The output should include "ui, enhancement"
      The output should include "[DRY-RUN] Simulação ativa. Ticket não publicado."
    End

    It 'closes the right border on all lines of the ASCII box'
      sample_tickets='{
  "tickets": [
    {
      "title": "fix(generator): fix ASCII table right border and text wrapping in columns",
      "rationale": "Separar a apresentação visual da tabela fechando a margem direita",
      "labels": ["bug", "generator"],
      "body": "## Description\nThis is a long line that demonstrates wrapping in the body preview section of the ticket box."
    }
  ]
}'
      When call _diva_present_and_dispatch "$sample_tickets" "codeberg" "lumenpink" "myproject" 1
      The status should be success
      The output should include "│ [ДИВАН] Ticket 1/1                                                          │"
      The output should include "│ 📌 Título   : fix(generator): fix ASCII table right border and text         │"
      The output should include "│               wrapping in columns                                           │"
      The output should include "│ 💡 Racional : Separar a apresentação visual da tabela fechando a margem     │"
      The output should include "│ 🏷️ Labels   : bug, generator                                                │"
      The output should include "│ 📝 Prévia do Corpo:                                                         │"
      The output should include "│    ## Description                                                           │"
    End

    It 'wraps long text across multiple lines with aligned continuation prefix'
      long_title="fix(generator): extremely long title that exceeds sixty-one characters by a very significant margin to trigger multi-line column text wrapping"
      sample_tickets=$(printf '{"tickets":[{"title":"%s","body":"preview"}]}' "$long_title")
      When call _diva_present_and_dispatch "$sample_tickets" "codeberg" "lumenpink" "myproject" 1
      The status should be success
      The output should include "│ 📌 Título   : fix(generator): extremely long title that exceeds sixty-one   │"
      The output should include "│               characters by a very significant margin to trigger            │"
      The output should include "│               multi-line column text wrapping                               │"
    End

    It 'displays full body of tall tickets without truncating at 8 lines'
      sample_tickets='{
  "tickets": [
    {
      "title": "fix: tall ticket test",
      "body": "Line 1: Problem statement\nLine 2: Context and background\nLine 3: Affected versions\nLine 4: Configuration settings\nLine 5: Step one to reproduce\nLine 6: Step two to reproduce\nLine 7: Step three to reproduce\nLine 8: Expected outcome\nLine 9: Actual observed outcome\nLine 10: Stack trace header\nLine 11: Error details\nLine 12: Workaround attempted\nLine 13: Proposed architecture fix\nLine 14: Acceptance criteria\nLine 15: Concluding remarks"
    }
  ]
}'
      When call _diva_present_and_dispatch "$sample_tickets" "codeberg" "lumenpink" "myproject" 1
      The status should be success
      The output should include "Line 1: Problem statement"
      The output should include "Line 8: Expected outcome"
      The output should include "Line 9: Actual observed outcome"
      The output should include "Line 12: Workaround attempted"
      The output should include "Line 15: Concluding remarks"
    End
  End

  Describe 'cmd_diva CLI interface'
    It 'displays help on --help'
      When call cmd_diva --help
      The status should be success
      The output should include "cccp.sh diva"
      The output should include "Options:"
    End

    It 'works via cmd_divan alias'
      When call cmd_divan --help
      The status should be success
      The output should include "cccp.sh diva"
    End

    It 'saves gemini_api_key via config subcommand'
      When call cmd_diva config "AIzaSyCustomKey789"
      The status should be success
      The output should include "gemini_api_key salva com sucesso"
      The contents of file "$TEST_DIR/.cccprc" should include "gemini_api_key = AIzaSyCustomKey789"
    End

    It 'fails with actionable error when GEMINI_API_KEY is missing'
      When call cmd_diva "Report text without API key"
      The status should be failure
      The error should include "GEMINI_API_KEY não configurada"
      The error should include "export GEMINI_API_KEY="
    End
  End

  Describe 'strict zero-awk compliance'
    It 'verifies zero executable awk invocations in src/utils/diva.sh'
      test_awk() {
        grep -rnw "awk" "$SHELLSPEC_PROJECT_ROOT/src/utils/diva.sh" | grep -v '#.*awk' || true
      }
      When call test_awk
      The status should be success
      The output should eq ""
    End
  End
End

# Include the test helper
. "$SHELLSPEC_PROJECT_ROOT/spec/spec_helper.sh"

Describe 'diva'
  Include "${CCCP_BUNDLE:-$SHELLSPEC_PROJECT_ROOT/src/utils/help.sh}"
  Include "${CCCP_BUNDLE:-$SHELLSPEC_PROJECT_ROOT/src/utils/config_manager.sh}"
  Include "${CCCP_BUNDLE:-$SHELLSPEC_PROJECT_ROOT/src/utils/repo.sh}"
  Include "${CCCP_BUNDLE:-$SHELLSPEC_PROJECT_ROOT/src/utils/catalog.sh}"
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

    It 'fails when -r specifies an unresolvable repo'
      export GEMINI_API_KEY="AIzaSyTestKey123"
      rm -rf "$TEST_DIR/.git"
      unset FORGEJO_OWNER CODEBERG_OWNER GITHUB_OWNER || true
      When call cmd_diva -r "unresolvable_repo" "Some issue text"
      The status should be failure
      The error should include "Repositório \"unresolvable_repo\" não encontrado"
    End
  End

  Describe '_diva_save_draft'
    It 'saves draft JSON file in .antigravity/diva_drafts/ and ensures gitignore'
      When call _diva_save_draft "forgejo" "lumenpink" "cccp" "fix(core): critical bug" "Line 1\nLine 2" "bug, core"
      The status should be success
      The error should include "Draft salvo em:"
      The path "$TEST_DIR/.antigravity/diva_drafts" should be directory
      The contents of file "$TEST_DIR/.gitignore" should include ".antigravity/"
    End
  End

  Describe '_diva_present_and_dispatch failure and draft preservation'
    It 'fails loudly to stderr and saves draft when forge is unsupported'
      sample_tickets='{"tickets":[{"title":"fix: unhandled forge","body":"body text","labels":["bug"]}]}'
      test_unsupported_forge() {
        _diva_present_and_dispatch "$sample_tickets" "unsupported_forge" "org" "repo" 0 "example.com" <<EOF
y
EOF
      }
      When call test_unsupported_forge
      The status should be success
      The output should include "Sessão concluída. 0 ticket(s) publicado(s)."
      The error should include "não suportada para publicação automática"
      The error should include "Draft salvo em:"
      The path "$TEST_DIR/.antigravity/diva_drafts" should be directory
    End

    It 'fails loudly to stderr and saves draft when forgejo token is missing'
      unset FORGEJO_TOKEN CODEBERG_TOKEN || true
      sample_tickets='{"tickets":[{"title":"fix: forgejo test","body":"body text","labels":["bug"]}]}'
      test_forgejo_no_token() {
        _diva_present_and_dispatch "$sample_tickets" "forgejo" "lumenpink" "cccp" 0 "git.lumenpink.com" <<EOF
y
EOF
      }
      When call test_forgejo_no_token
      The status should be success
      The output should include "Sessão concluída. 0 ticket(s) publicado(s)."
      The error should include "Token para Forgejo (git.lumenpink.com) não encontrado"
      The error should include "Draft salvo em:"
      The path "$TEST_DIR/.antigravity/diva_drafts" should be directory
    End
  End

  Describe '_diva_present_and_dispatch manual title and body editing'
    It 'allows editing ticket title interactively before publishing'
      sample_tickets='{"tickets":[{"title":"fix: initial title","body":"ticket body","labels":["bug"]}]}'
      export CODEBERG_TOKEN="fake-token"
      curl() {
        printf '{"html_url":"https://codeberg.org/lumenpink/cccp/issues/99"}\n'
      }
      test_edit_title() {
        _diva_present_and_dispatch "$sample_tickets" "codeberg" "lumenpink" "cccp" 0 "codeberg.org" <<'EOF'
t
fix: edited title
y
EOF
      }
      When call test_edit_title
      The status should be success
      The output should include "Título atualizado."
      The output should include "fix: edited title"
      The output should include "Criado: https://codeberg.org/lumenpink/cccp/issues/99"
    End

    It 'allows editing ticket body interactively via EDITOR before publishing'
      sample_tickets='{"tickets":[{"title":"fix: test ticket","body":"original body","labels":["bug"]}]}'
      export CODEBERG_TOKEN="fake-token"
      mock_editor="$TEST_DIR/mock_editor.sh"
      cat > "$mock_editor" << 'EOF'
#!/bin/sh
echo "edited body via mock editor" > "$1"
EOF
      chmod +x "$mock_editor"
      export EDITOR="$mock_editor"

      curl() {
        printf '{"html_url":"https://codeberg.org/lumenpink/cccp/issues/100"}\n'
      }
      test_edit_body() {
        _diva_present_and_dispatch "$sample_tickets" "codeberg" "lumenpink" "cccp" 0 "codeberg.org" <<'EOF'
b
y
EOF
      }
      When call test_edit_body
      The status should be success
      The output should include "Corpo atualizado."
      The output should include "edited body via mock editor"
      The output should include "Criado: https://codeberg.org/lumenpink/cccp/issues/100"
    End

    It 'allows editing ticket body using kv when present in PATH'
      sample_tickets='{"tickets":[{"title":"fix: kv test","body":"original body","labels":["bug"]}]}'
      export CODEBERG_TOKEN="fake-token"
      unset VISUAL EDITOR DIVA_EDITOR || true
      mkdir -p "$TEST_DIR/bin"
      cat > "$TEST_DIR/bin/kv" << 'EOF'
#!/bin/sh
echo "edited body via kv editor" > "$1"
EOF
      chmod +x "$TEST_DIR/bin/kv"
      export PATH="$TEST_DIR/bin:$PATH"

      curl() {
        printf '{"html_url":"https://codeberg.org/lumenpink/cccp/issues/101"}\n'
      }
      test_edit_kv() {
        _diva_present_and_dispatch "$sample_tickets" "codeberg" "lumenpink" "cccp" 0 "codeberg.org" <<'EOF'
b
y
EOF
      }
      When call test_edit_kv
      The status should be success
      The output should include "Abrindo editor (kv)"
      The output should include "Corpo atualizado."
      The output should include "edited body via kv editor"
      The output should include "Criado: https://codeberg.org/lumenpink/cccp/issues/101"
    End
  End

  Describe '_diva_resolve_editor'
    It 'prioritizes DIVA_EDITOR environment variable'
      export DIVA_EDITOR="custom_editor"
      When call _diva_resolve_editor
      The status should be success
      The output should eq "custom_editor"
    End

    It 'reads diva_editor from .cccprc when DIVA_EDITOR is unset'
      unset DIVA_EDITOR || true
      echo "diva_editor = my_preferred_editor" > "$TEST_DIR/.cccprc"
      When call _diva_resolve_editor
      The status should be success
      The output should eq "my_preferred_editor"
    End

    It 'falls back to VISUAL or EDITOR when no diva config exists'
      unset DIVA_EDITOR || true
      rm -f "$TEST_DIR/.cccprc"
      export VISUAL="my_visual_editor"
      When call _diva_resolve_editor
      The status should be success
      The output should eq "my_visual_editor"
    End
  End

  Describe '_diva_resolve_custom_repo'
    It 'resolves full owner/repo slug directly'
      When call _diva_resolve_custom_repo "lumenpink/myproject"
      The status should be success
      The output should eq "forgejo lumenpink myproject "
    End

    It 'resolves partial repo name from catalog.json'
      cat > "$TEST_DIR/catalog.json" << 'CATJSON'
[
  {
    "name": "supertool",
    "forge": "github",
    "slug": "acme/supertool"
  }
]
CATJSON
      When call _diva_resolve_custom_repo "supertool"
      The status should be success
      The output should eq "github acme supertool github.com"
    End

    It 'resolves partial repo name using active repository owner fallback'
      When call _diva_resolve_custom_repo "otherproj"
      The status should be success
      The output should eq "forgejo lumenpink otherproj codeberg.org"
    End

    It 'detects ambiguity and fails when multiple catalog entries match the partial name'
      cat > "$TEST_DIR/catalog.json" << 'CATJSON'
[
  {
    "name": "sharedlib",
    "forge": "github",
    "slug": "alpha/sharedlib"
  },
  {
    "name": "sharedlib",
    "forge": "codeberg",
    "slug": "beta/sharedlib"
  }
]
CATJSON
      When call _diva_resolve_custom_repo "sharedlib"
      The status should be failure
      The error should include "O nome de repositório \"sharedlib\" é ambíguo"
      The error should include "alpha/sharedlib (github)"
      The error should include "beta/sharedlib (codeberg)"
    End

    It 'fails when repo cannot be resolved and no owner exists'
      rm -rf "$TEST_DIR/.git"
      unset FORGEJO_OWNER CODEBERG_OWNER GITHUB_OWNER || true
      test_unresolvable() {
        _diva_resolve_custom_repo "completely_unknown_repo"
      }
      When call test_unresolvable
      The status should be failure
      The error should include "Repositório \"completely_unknown_repo\" não encontrado"
    End
  End

  Describe '_diva_call_gemini parameter and refinement'
    It 'passes custom temperature and refinement instructions in the request payload'
      test_gemini_call() {
        curl() {
          while [ $# -gt 0 ]; do
            case "$1" in
              -d)
                shift
                printf '%s' "$1" > "$TEST_DIR/curl_captured.json"
                shift
                ;;
              *)
                shift
                ;;
            esac
          done
          printf '{"candidates":[{"content":{"parts":[{"text":"{\"tickets\":[]}"}]}}]}'
        }
        _diva_call_gemini "fake_key" "gemini-3.6-flash" "problema no form" "lumenpink/cccp (codeberg)" "0.8" "divida em dois tickets" '{"tickets":[{"title":"old"}]}' >/dev/null
        grep -q '"temperature": 0.8' "$TEST_DIR/curl_captured.json" || return 1
        grep -q 'divida em dois tickets' "$TEST_DIR/curl_captured.json" || return 1
        grep -q 'problema no form' "$TEST_DIR/curl_captured.json" || return 1
        return 0
      }
      When call test_gemini_call
      The status should be success
    End
  End

  Describe '_diva_present_and_dispatch interactive refinement'
    It 'allows rediscutir with IA using option r, updates temperature and restarts triage presentation'
      test_rediscutir() {
        export CODEBERG_TOKEN="mock_codeberg_token"
        _diva_call_gemini() {
          printf '{"candidates":[{"content":{"parts":[{"text":"{\"tickets\":[{\"title\":\"feat(refined): new atomic ticket\",\"body\":\"Refined body\"}]}"}]}}]}'
        }
        curl() {
          printf '{"html_url":"https://codeberg.org/lumenpink/cccp/issues/99"}'
        }
        sample_tickets='{"tickets":[{"title":"fix: old initial ticket","body":"Old body"}]}'
        _diva_present_and_dispatch "$sample_tickets" "codeberg" "lumenpink" "cccp" 0 "codeberg.org" "fake_api_key" "gemini-3.6-flash" "relato inicial" 0.2 <<'EOF'
r
:temp 0.9 refazer com mais detalhes
y
EOF
      }
      When call test_rediscutir
      The status should be success
      The output should include "Consultando Gemini (gemini-3.6-flash, temp=0.9) com refinamento..."
      The output should include "Proposta refinada com sucesso pela Gemini (1 ticket(s))"
      The output should include "feat(refined): new atomic ticket"
      The output should include "✅ Criado: https://codeberg.org/lumenpink/cccp/issues/99"
    End
  End

  Describe 'cmd_diva CLI option parsing'
    It 'accepts --temp argument in cmd_diva'
      test_diva_temp() {
        _diva_resolve_api_key() { echo "fake_key"; }
        _diva_resolve_forge() { echo "forgejo lumenpink cccp codeberg.org"; }
        _diva_call_gemini() {
          echo "TEMP_CALLED_WITH:$5" >&2
          printf '{"candidates":[{"content":{"parts":[{"text":"{\"tickets\":[]}"}]}}]}'
        }
        cmd_diva --dry-run --temp 0.7 "some report" 2>&1 || true
      }
      When call test_diva_temp
      The output should include "temp=0.7"
    End
  End

  Describe '_diva_extract_html_url'
    It 'extracts html_url using jq when available'
      sample_json='{"id":123,"html_url":"https://codeberg.org/lumenpink/cccp/issues/42","state":"open"}'
      When call _diva_extract_html_url "$sample_json"
      The status should be success
      The output should eq "https://codeberg.org/lumenpink/cccp/issues/42"
    End

    It 'extracts html_url in pure POSIX mode when jq is unavailable'
      test_posix_extract() {
        jq() { return 127; }
        sample_json='{"id":456,"html_url":"https://github.com/lumenpink/kvant/issues/1","title":"test"}'
        _diva_extract_html_url "$sample_json"
      }
      When call test_posix_extract
      The status should be success
      The output should eq "https://github.com/lumenpink/kvant/issues/1"
    End
  End

  Describe 'cmd_diva web option'
    It 'dispatches to cmd_web on --web flag'
      cmd_web() {
        echo "MOCK_CMD_WEB_CALLED: $@"
        return 0
      }
      When call cmd_diva --web --no-open
      The status should be success
      The output should include "MOCK_CMD_WEB_CALLED: diva --no-open"
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

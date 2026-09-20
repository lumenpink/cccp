# Include the test helper
. "$SHELLSPEC_PROJECT_ROOT/spec/spec_helper.sh"

Describe "codeberg"
    Include "${CCCP_BUNDLE:-$SHELLSPEC_PROJECT_ROOT/src/utils/config_manager.sh}"
    Include "${CCCP_BUNDLE:-$SHELLSPEC_PROJECT_ROOT/src/utils/tickets.sh}"
    Include "${CCCP_BUNDLE:-$SHELLSPEC_PROJECT_ROOT/src/utils/agents.sh}"
    Include "${CCCP_BUNDLE:-$SHELLSPEC_PROJECT_ROOT/src/utils/codeberg.sh}"

    setup() {
        TEST_DIR="$(mktemp -d)"
        cd "$TEST_DIR"
        git init -q -b main
        git config user.email "test@example.com"
        git config user.name "Test User"
        echo "init" > README.md
        git add README.md
        git commit -q -m "chore: initial commit"
        GIT_ROOT="$TEST_DIR"
    }

    cleanup() {
        cd /
        rm -rf "$TEST_DIR"
    }

    BeforeEach "setup"
    AfterEach "cleanup"

    Describe "parse_codeberg_url"
        It "parses SSH codeberg URL"
            When call parse_codeberg_url "git@codeberg.org:lumenpink/cccp.git"
            The output should eq "lumenpink cccp"
            The status should be success
        End

        It "parses HTTPS codeberg URL"
            When call parse_codeberg_url "https://codeberg.org/lumenpink/cccp.git"
            The output should eq "lumenpink cccp"
            The status should be success
        End

        It "parses HTTPS codeberg URL without .git"
            When call parse_codeberg_url "https://codeberg.org/lumenpink/cccp"
            The output should eq "lumenpink cccp"
            The status should be success
        End

        It "returns failure for non-codeberg URLs"
            When call parse_codeberg_url "https://github.com/lumenpink/cccp.git"
            The status should be failure
        End
    End

    Describe "cmd_codeberg setup"
        It "aborts when run outside a git repository"
            NOGIT_DIR="$(mktemp -d)"
            cd "$NOGIT_DIR"
            When call cmd_codeberg setup
            cd "$TEST_DIR"
            rm -rf "$NOGIT_DIR"
            The stderr should include "Not inside a git repository"
            The status should be failure
        End

        It "aborts when no origin remote is configured"
            When call cmd_codeberg setup
            The stderr should include "No 'origin' remote"
            The status should be failure
        End

        It "aborts when remote is not Codeberg"
            git remote add origin https://github.com/user/repo.git
            When call cmd_codeberg setup
            The stderr should include "does not appear to be a Codeberg"
            The status should be failure
        End

        It "saves codeberg_owner and codeberg_repo to .cccprc"
            git remote add origin https://codeberg.org/testowner/testrepo.git
            fetch_agents_template() { printf '# Template\nNo vars here.\n'; }
            extract_template_vars() { printf ''; }
            When call cmd_codeberg setup
            The output should include "Codeberg setup complete"
            The file "$TEST_DIR/.cccprc" should be exist
            The contents of file "$TEST_DIR/.cccprc" should include "codeberg_owner"
            The contents of file "$TEST_DIR/.cccprc" should include "testowner"
            The contents of file "$TEST_DIR/.cccprc" should include "codeberg_repo"
            The contents of file "$TEST_DIR/.cccprc" should include "testrepo"
            The status should be success
        End

        It "enables ticket in post_push_sync"
            git remote add origin https://codeberg.org/testowner/testrepo.git
            fetch_agents_template() { printf '# Template\nNo vars here.\n'; }
            extract_template_vars() { printf ''; }
            When call cmd_codeberg setup
            The output should include "Enabled post_push_sync = ticket"
            The contents of file "$TEST_DIR/.cccprc" should include "post_push_sync"
            The contents of file "$TEST_DIR/.cccprc" should include "ticket"
            The status should be success
        End

        It "enables agent in post_push_sync"
            git remote add origin https://codeberg.org/testowner/testrepo.git
            fetch_agents_template() { printf '# Template\nNo vars here.\n'; }
            extract_template_vars() { printf ''; }
            When call cmd_codeberg setup
            The output should include "Enabled post_push_sync = agent"
            The contents of file "$TEST_DIR/.cccprc" should include "agent"
            The status should be success
        End

        It "skips prompting for already-configured variables"
            git remote add origin https://codeberg.org/testowner/testrepo.git
            fetch_agents_template() { printf '# Template\n${AGENTS_LANG}\n'; }
            extract_template_vars() { printf 'AGENTS_LANG'; }
            resolve_agent_var() { printf 'english'; return 0; }
            When call cmd_codeberg setup
            The output should include "already set, skipping"
            The status should be success
        End

        It "saves the template URL to .cccprc"
            git remote add origin https://codeberg.org/testowner/testrepo.git
            fetch_agents_template() { printf '# Template\nNo vars.\n'; }
            extract_template_vars() { printf ''; }
            When call cmd_codeberg setup --url "https://example.com/template.md"
            The output should include "Using AGENTS template: https://example.com/template.md"
            The contents of file "$TEST_DIR/.cccprc" should include "agent_md_url"
            The contents of file "$TEST_DIR/.cccprc" should include "https://example.com/template.md"
            The status should be success
        End

        It "appends agent to existing post_push_sync value"
            git remote add origin https://codeberg.org/testowner/testrepo.git
            fetch_agents_template() { printf '# Template\nNo vars.\n'; }
            extract_template_vars() { printf ''; }
            write_file_key "$TEST_DIR/.cccprc" "post_push_sync" "ticket"
            When call cmd_codeberg setup
            The output should include "Codeberg setup complete"
            The contents of file "$TEST_DIR/.cccprc" should include "ticket"
            The contents of file "$TEST_DIR/.cccprc" should include "agent"
            The status should be success
        End
    End
End

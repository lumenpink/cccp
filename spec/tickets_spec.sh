# Include the test helper
. "$SHELLSPEC_PROJECT_ROOT/spec/spec_helper.sh"

Describe 'tickets'
  Include "${CCCP_BUNDLE:-$SHELLSPEC_PROJECT_ROOT/src/utils/help.sh}"
  Include "${CCCP_BUNDLE:-$SHELLSPEC_PROJECT_ROOT/src/utils/config_manager.sh}"
  Include "${CCCP_BUNDLE:-$SHELLSPEC_PROJECT_ROOT/src/utils/tickets.sh}"

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
    unset CODEBERG_OWNER || true
    unset CODEBERG_REPO || true
    unset CODEBERG_USER || true
    unset CODEBERG_TOKEN || true
    unset TICKETS_FILE || true
    unset CODEBERG_TICKETS_FILE || true
  }

  cleanup() {
    rm -rf "$TEST_DIR"
  }

  BeforeEach 'setup'
  AfterEach 'cleanup'

  Describe 'parse_codeberg_url'
    It 'parses SSH codeberg URL'
      When call parse_codeberg_url "git@codeberg.org:alice/wonderland.git"
      The status should be success
      The output should eq "alice wonderland"
    End

    It 'parses HTTPS codeberg URL'
      When call parse_codeberg_url "https://codeberg.org/bob/builder.git"
      The status should be success
      The output should eq "bob builder"
    End

    It 'parses HTTPS codeberg URL without .git'
      When call parse_codeberg_url "https://codeberg.org/carol/crypto"
      The status should be success
      The output should eq "carol crypto"
    End

    It 'parses ssh:// protocol codeberg URL'
      When call parse_codeberg_url "ssh://git@codeberg.org/dave/space.git"
      The status should be success
      The output should eq "dave space"
    End

    It 'returns failure for non-codeberg URLs'
      When call parse_codeberg_url "git@github.com:someone/project.git"
      The status should be failure
    End
  End

  Describe 'resolve_codeberg_info'
    It 'prioritizes CODEBERG_OWNER and CODEBERG_REPO'
      export CODEBERG_OWNER="myorg"
      export CODEBERG_REPO="myrepo"
      git remote add origin "git@codeberg.org:other/repo.git"

      When call resolve_codeberg_info
      The status should be success
      The output should eq "myorg myrepo"
    End

    It 'resolves from git remote origin when env is unset'
      git remote add origin "git@codeberg.org:team/awesome.git"

      When call resolve_codeberg_info
      The status should be success
      The output should eq "team awesome"
    End

    It 'fails when no codeberg remote or env exists'
      git remote add origin "git@github.com:team/awesome.git"

      When call resolve_codeberg_info
      The status should be failure
    End
  End

  Describe 'cmd_tickets'
    It 'shows usage on --help flag'
      When call cmd_tickets --help
      The status should be success
      The output should include "cccp.sh tickets - Synchronize Codeberg Issues into Markdown"
      The output should include "Usage:"
    End

    It 'exits 0 with warning on stderr when no Codeberg repo is found'
      git remote add origin "git@github.com:foo/bar.git"

      test_no_codeberg() {
        cmd_tickets
      }

      When call test_no_codeberg
      The status should be success
      The error should include "Warning: Unable to determine Codeberg repository and owner"
    End

    It 'ensures tickets file directory is in .gitignore'
      export CODEBERG_OWNER="testowner"
      export CODEBERG_REPO="testrepo"

      # Mock curl to return an empty JSON array
      curl() {
        echo "[]"
      }

      test_gitignore_creation() {
        cmd_tickets >/dev/null 2>&1
        cat "$TEST_DIR/.gitignore" 2>/dev/null || true
      }

      When call test_gitignore_creation
      The status should be success
      The output should include ".antigravity/"
    End

    It 'syncs tickets formatting and respects budget ceiling'
      export CODEBERG_OWNER="testowner"
      export CODEBERG_REPO="testrepo"

      curl() {
        cat << 'JSON'
[
  {
    "number": 1,
    "title": "First Ticket",
    "state": "open",
    "labels": [{"name": "bug"}],
    "html_url": "https://codeberg.org/testowner/testrepo/issues/1",
    "body": "First issue body content."
  },
  {
    "number": 2,
    "title": "Second Ticket",
    "state": "open",
    "labels": [],
    "html_url": "https://codeberg.org/testowner/testrepo/issues/2",
    "body": "Second issue body content."
  }
]
JSON
      }

      target="$TEST_DIR/custom_tickets.md"
      test_sync_tickets() {
        cmd_tickets --file "$target"
      }

      When call test_sync_tickets
      The status should be success
      The output should include "Done! Synced 2 complete ticket(s)"
      The path "$target" should be exist
    End

    It 'exits 0 with warning when Codeberg API call fails'
      export CODEBERG_OWNER="testowner"
      export CODEBERG_REPO="testrepo"

      curl() {
        return 1
      }

      test_api_fail() {
        cmd_tickets
      }

      When call test_api_fail
      The status should be success
      The output should include "Fetching open issues from Codeberg"
      The error should include "Warning: Failed to retrieve or parse issues from Codeberg API"
    End
  End
End

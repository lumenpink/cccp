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
    unset TICKETS_PROVIDER || true
    unset TICKETS_OWNER || true
    unset TICKETS_REPO || true
    unset TICKETS_TOKEN || true
    unset CODEBERG_OWNER || true
    unset CODEBERG_REPO || true
    unset CODEBERG_USER || true
    unset CODEBERG_TOKEN || true
    unset GITHUB_OWNER || true
    unset GITHUB_REPO || true
    unset GITHUB_USER || true
    unset GITHUB_TOKEN || true
    unset TICKETS_FILE || true
    unset CODEBERG_TICKETS_FILE || true
    unset TICKETS_HOURLY_LIMIT || true
    unset TICKETS_RATE_LIMIT_FILE || true
  }

  cleanup() {
    rm -rf "$TEST_DIR"
  }

  BeforeEach 'setup'
  AfterEach 'cleanup'

  Describe 'parse_remote_url'
    It 'parses SSH GitHub URL'
      When call parse_remote_url "git@github.com:alice/wonderland.git"
      The status should be success
      The output should eq "github.com alice wonderland"
    End

    It 'parses HTTPS GitHub URL'
      When call parse_remote_url "https://github.com/bob/builder.git"
      The status should be success
      The output should eq "github.com bob builder"
    End

    It 'parses HTTPS GitHub URL without .git'
      When call parse_remote_url "https://github.com/carol/crypto"
      The status should be success
      The output should eq "github.com carol crypto"
    End

    It 'parses SSH Codeberg URL'
      When call parse_remote_url "git@codeberg.org:alice/wonderland.git"
      The status should be success
      The output should eq "codeberg.org alice wonderland"
    End

    It 'parses HTTPS Codeberg URL'
      When call parse_remote_url "https://codeberg.org/bob/builder.git"
      The status should be success
      The output should eq "codeberg.org bob builder"
    End

    It 'parses SSH Framagit URL'
      When call parse_remote_url "git@framagit.org:myteam/myrepo.git"
      The status should be success
      The output should eq "framagit.org myteam myrepo"
    End

    It 'parses HTTPS GitLab URL with subgroups'
      When call parse_remote_url "https://gitlab.com/group/subgroup/repo.git"
      The status should be success
      The output should eq "gitlab.com group/subgroup repo"
    End

    It 'parses SSH SourceHut URL'
      When call parse_remote_url "git@git.sr.ht:~alice/wonderland"
      The status should be success
      The output should eq "git.sr.ht ~alice wonderland"
    End

    It 'returns failure for invalid URLs'
      When call parse_remote_url "invalid-url"
      The status should be failure
    End
  End

  Describe 'detect_forge_provider'
    It 'maps github.com to github'
      When call detect_forge_provider "github.com"
      The status should be success
      The output should eq "github"
    End

    It 'maps codeberg.org to forgejo'
      When call detect_forge_provider "codeberg.org"
      The status should be success
      The output should eq "forgejo"
    End

    It 'maps framagit.org to gitlab'
      When call detect_forge_provider "framagit.org"
      The status should be success
      The output should eq "gitlab"
    End

    It 'maps git.sr.ht to sourcehut'
      When call detect_forge_provider "git.sr.ht"
      The status should be success
      The output should eq "sourcehut"
    End

    It 'actively probes self-hosted Forgejo instance'
      curl() {
        echo '{"version": "7.0.4"}'
      }
      When call detect_forge_provider "git.customforge.net"
      The status should be success
      The output should eq "forgejo"
    End

    It 'actively probes self-hosted GitLab instance'
      curl() {
        if [ "$1" = "-s" ] && [ "$2" = "-f" ] && [ "$3" = "-m" ]; then
          return 1
        fi
        echo "HTTP/2 200\r\nserver: nginx\r\nx-gitlab-meta: ok\r\n"
      }
      When call detect_forge_provider "gitlab.mycompany.internal"
      The status should be success
      The output should eq "gitlab"
    End

    It 'fails when custom host probe cannot be detected'
      curl() {
        return 1
      }
      When call detect_forge_provider "git.unknown.xyz"
      The status should be failure
    End
  End

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

  Describe 'parse_github_url'
    It 'parses SSH github URL'
      When call parse_github_url "git@github.com:alice/wonderland.git"
      The status should be success
      The output should eq "alice wonderland"
    End

    It 'parses HTTPS github URL'
      When call parse_github_url "https://github.com/bob/builder.git"
      The status should be success
      The output should eq "bob builder"
    End

    It 'returns failure for non-github URLs'
      When call parse_github_url "git@codeberg.org:someone/project.git"
      The status should be failure
    End
  End

  Describe 'parse_gitlab_url'
    It 'parses Framagit URL'
      When call parse_gitlab_url "git@framagit.org:alice/wonderland.git"
      The status should be success
      The output should eq "alice wonderland"
    End

    It 'parses GitLab URL'
      When call parse_gitlab_url "https://gitlab.com/bob/builder.git"
      The status should be success
      The output should eq "bob builder"
    End

    It 'returns failure for non-gitlab URLs'
      When call parse_gitlab_url "git@github.com:someone/project.git"
      The status should be failure
    End
  End

  Describe 'resolve_tickets_info'
    It 'resolves GitHub remote origin'
      git remote add origin "git@github.com:lumenpink/cccp.git"

      When call resolve_tickets_info
      The status should be success
      The output should eq "github lumenpink cccp github.com"
    End

    It 'resolves Codeberg remote origin'
      git remote add origin "git@codeberg.org:lumenpink/cccp.git"

      When call resolve_tickets_info
      The status should be success
      The output should eq "forgejo lumenpink cccp codeberg.org"
    End

    It 'resolves Framagit remote origin'
      git remote add origin "git@framagit.org:myteam/myrepo.git"

      When call resolve_tickets_info
      The status should be success
      The output should eq "gitlab myteam myrepo framagit.org"
    End

    It 'prioritizes remote pushurl over fetch url'
      git remote add origin "git@codeberg.org:fetch/repo.git"
      git config remote.origin.pushurl "git@github.com:push/repo.git"

      When call resolve_tickets_info
      The status should be success
      The output should eq "github push repo github.com"
    End

    It 'prioritizes TICKETS_OWNER and TICKETS_REPO'
      export TICKETS_OWNER="myorg"
      export TICKETS_REPO="myrepo"
      export TICKETS_PROVIDER="github"
      git remote add origin "git@codeberg.org:other/repo.git"

      When call resolve_tickets_info
      The status should be success
      The output should eq "github myorg myrepo github.com"
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

  Describe 'check_tickets_rate_limit'
    It 'allows calls when under the hourly limit'
      export TICKETS_RATE_LIMIT_FILE="$TEST_DIR/rate_limit.txt"
      export TICKETS_HOURLY_LIMIT=3

      When call check_tickets_rate_limit
      The status should be success
      The file "$TEST_DIR/rate_limit.txt" should be exist
    End

    It 'gracefully blocks and warns when limit is reached'
      export TICKETS_RATE_LIMIT_FILE="$TEST_DIR/rate_limit.txt"
      export TICKETS_HOURLY_LIMIT=2
      now=$(date +%s)
      printf '%s\n%s\n' "$now" "$now" > "$TEST_DIR/rate_limit.txt"

      When call check_tickets_rate_limit
      The status should be failure
      The error should include "Gosplan Quota"
      The error should include "Hourly limit reached"
    End

    It 'expires timestamps older than 1 hour'
      export TICKETS_RATE_LIMIT_FILE="$TEST_DIR/rate_limit.txt"
      export TICKETS_HOURLY_LIMIT=2
      now=$(date +%s)
      old=$((now - 4000))
      printf '%s\n%s\n' "$old" "$old" > "$TEST_DIR/rate_limit.txt"

      When call check_tickets_rate_limit
      The status should be success
    End
  End

  Describe 'cmd_tickets'
    It 'shows usage on --help flag'
      When call cmd_tickets --help
      The status should be success
      The output should include "cccp.sh tickets - Synchronize"
      The output should include "Usage:"
    End

    It 'exits 0 with warning on stderr when no supported repo is found'
      git remote add origin "git@unknown-host.xyz:foo/bar.git"

      curl() {
        return 1
      }

      test_no_repo() {
        cmd_tickets
      }

      When call test_no_repo
      The status should be success
      The error should include "Warning: Unable to determine repository and owner"
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

    It 'syncs Codeberg tickets formatting and respects budget ceiling'
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

    It 'syncs GitHub tickets and filters out pull requests'
      git remote add origin "git@github.com:octocat/Hello-World.git"

      curl() {
        cat << 'JSON'
[
  {
    "number": 10,
    "title": "A Real Bug Issue",
    "state": "open",
    "labels": [{"name": "defect"}],
    "html_url": "https://github.com/octocat/Hello-World/issues/10",
    "body": "Bug description here.",
    "pull_request": null
  },
  {
    "number": 11,
    "title": "Feature PR that must be skipped",
    "state": "open",
    "labels": [{"name": "enhancement"}],
    "html_url": "https://github.com/octocat/Hello-World/pull/11",
    "body": "PR description here.",
    "pull_request": {
      "url": "https://api.github.com/repos/octocat/Hello-World/pulls/11"
    }
  },
  {
    "number": 12,
    "title": "Another Genuine Issue",
    "state": "open",
    "labels": [],
    "html_url": "https://github.com/octocat/Hello-World/issues/12",
    "body": "Another issue description."
  }
]
JSON
      }

      target="$TEST_DIR/gh_tickets.md"
      test_gh_sync() {
        cmd_tickets --file "$target"
      }

      When call test_gh_sync
      The status should be success
      The output should include "Fetching open issues from GitHub"
      The output should include "Done! Synced 2 complete ticket(s)"
      The contents of file "$target" should include "A Real Bug Issue"
      The contents of file "$target" should include "Another Genuine Issue"
      The contents of file "$target" should not include "Feature PR that must be skipped"
    End

    It 'syncs GitLab/Framagit tickets and normalizes fields'
      git remote add origin "git@framagit.org:myorg/myproject.git"

      curl() {
        cat << 'JSON'
[
  {
    "iid": 42,
    "title": "GitLab Genuine Issue",
    "state": "opened",
    "labels": ["bug", "urgent"],
    "web_url": "https://framagit.org/myorg/myproject/-/issues/42",
    "description": "GitLab issue markdown body."
  }
]
JSON
      }

      target="$TEST_DIR/gl_tickets.md"
      test_gl_sync() {
        cmd_tickets --file "$target"
      }

      When call test_gl_sync
      The status should be success
      The output should include "Fetching open issues from Framagit"
      The output should include "Done! Synced 1 complete ticket(s)"
      The contents of file "$target" should include "GitLab Genuine Issue"
      The contents of file "$target" should include "bug, urgent"
    End

    It 'gracefully exits 0 when rate limit is hit in cmd_tickets'
      git remote add origin "git@github.com:octocat/Hello-World.git"
      export TICKETS_RATE_LIMIT_FILE="$TEST_DIR/rate_limit.txt"
      export TICKETS_HOURLY_LIMIT=1
      now=$(date +%s)
      echo "$now" > "$TEST_DIR/rate_limit.txt"

      When call cmd_tickets
      The status should be success
      The error should include "Gosplan Quota"
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

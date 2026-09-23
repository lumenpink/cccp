# Include the test helper
. "$SHELLSPEC_PROJECT_ROOT/spec/spec_helper.sh"

Describe 'repo.sh repository and remote resolver'
  Include "${CCCP_BUNDLE:-$SHELLSPEC_PROJECT_ROOT/src/utils/config_manager.sh}"
  Include "${CCCP_BUNDLE:-$SHELLSPEC_PROJECT_ROOT/src/utils/repo.sh}"

  setup() {
    TEST_DIR="$(mktemp -d)"
    cd "$TEST_DIR"
    git init -q
    git config user.email "test@example.com"
    git config user.name "Test User"
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

    It 'parses SSH URL with custom port'
      When call parse_remote_url "ssh://git@codeberg.org:2222/dave/space.git"
      The status should be success
      The output should eq "codeberg.org dave space"
    End

    It 'returns failure for invalid URLs'
      When call parse_remote_url "invalid-url"
      The status should be failure
    End

    It 'returns failure for empty URL'
      When call parse_remote_url ""
      The status should be failure
    End
  End

  Describe 'specific forge parsers'
    It 'parses codeberg URLs'
      When call parse_codeberg_url "git@codeberg.org:lumenpink/cccp.git"
      The status should be success
      The output should eq "lumenpink cccp"
    End

    It 'rejects non-codeberg URLs in parse_codeberg_url'
      When call parse_codeberg_url "https://github.com/lumenpink/cccp.git"
      The status should be failure
    End

    It 'parses github URLs'
      When call parse_github_url "https://github.com/octocat/Hello-World.git"
      The status should be success
      The output should eq "octocat Hello-World"
    End

    It 'rejects non-github URLs in parse_github_url'
      When call parse_github_url "https://codeberg.org/lumenpink/cccp.git"
      The status should be failure
    End

    It 'parses gitlab URLs'
      When call parse_gitlab_url "https://gitlab.com/group/myrepo.git"
      The status should be success
      The output should eq "group myrepo"
    End
  End

  Describe 'detect_forge_provider'
    It 'identifies github from domain'
      When call detect_forge_provider "github.com"
      The status should be success
      The output should eq "github"
    End

    It 'identifies forgejo from codeberg.org'
      When call detect_forge_provider "codeberg.org"
      The status should be success
      The output should eq "forgejo"
    End

    It 'identifies tildegit from tildegit.org'
      When call detect_forge_provider "tildegit.org"
      The status should be success
      The output should eq "tildegit"
    End

    It 'identifies gitlab from gitlab.com'
      When call detect_forge_provider "gitlab.com"
      The status should be success
      The output should eq "gitlab"
    End

    It 'identifies sourcehut from sr.ht'
      When call detect_forge_provider "git.sr.ht"
      The status should be success
      The output should eq "sourcehut"
    End
  End

  Describe 'detect_repo_info and accessors'
    It 'detects repository info from origin remote'
      git remote add origin "ssh://git@codeberg.org/lumenpink/cccp.git"
      When call detect_repo_info
      The status should be success
      The output should eq "forgejo lumenpink cccp codeberg.org"
    End

    It 'detects repository info from a specific path via argument'
      other_dir="$(mktemp -d)"
      git -C "$other_dir" init -q
      git -C "$other_dir" remote add origin "https://github.com/torvalds/linux.git"
      When call detect_repo_info "$other_dir"
      rm -rf "$other_dir"
      The status should be success
      The output should eq "github torvalds linux github.com"
    End

    It 'returns failure when not a git repo or no remote configured'
      When call detect_repo_info
      The status should be failure
    End

    It 'extracts slug via get_repo_slug'
      git remote add origin "https://github.com/torvalds/linux.git"
      When call get_repo_slug
      The status should be success
      The output should eq "torvalds/linux"
    End

    It 'extracts owner via get_repo_owner'
      git remote add origin "https://github.com/torvalds/linux.git"
      When call get_repo_owner
      The status should be success
      The output should eq "torvalds"
    End

    It 'extracts name via get_repo_name'
      git remote add origin "https://github.com/torvalds/linux.git"
      When call get_repo_name
      The status should be success
      The output should eq "linux"
    End

    It 'extracts forge via get_repo_forge'
      git remote add origin "https://github.com/torvalds/linux.git"
      When call get_repo_forge
      The status should be success
      The output should eq "github"
    End
  End

  Describe 'get_git_root'
    It 'returns git repository root path'
      When call get_git_root
      The status should be success
      The output should eq "$TEST_DIR"
    End
  End

  Describe 'ensure_gitignored'
    It 'adds entry to .gitignore'
      touch "$TEST_DIR/secret.txt"
      When call ensure_gitignored "$TEST_DIR/secret.txt"
      The status should be success
      The file "$TEST_DIR/.gitignore" should be exist
      The contents of file "$TEST_DIR/.gitignore" should include "secret.txt"
    End

    It 'adds .antigravity/ directory to .gitignore when target file is inside .antigravity'
      mkdir -p "$TEST_DIR/.antigravity"
      touch "$TEST_DIR/.antigravity/state.json"
      When call ensure_gitignored "$TEST_DIR/.antigravity/state.json"
      The status should be success
      The contents of file "$TEST_DIR/.gitignore" should include ".antigravity/"
    End

    It 'is idempotent and avoids duplicate entries in .gitignore'
      touch "$TEST_DIR/data.csv"
      ensure_gitignored "$TEST_DIR/data.csv"
      When call ensure_gitignored "$TEST_DIR/data.csv"
      The status should be success
      The contents of file "$TEST_DIR/.gitignore" should eq "data.csv"
    End
  End

  Describe 'escape_json_string'
    It 'escapes quotes, backslashes, tabs, and carriage returns'
      When call escape_json_string 'Line with "quotes" and \backslash and	tab'
      The status should be success
      The output should eq 'Line with \"quotes\" and \\backslash and\ttab'
    End
  End

  Describe 'resolve_forge_token'
    It 'resolves token from environment variable'
      export CODEBERG_TOKEN="env-token-12345"
      When call resolve_forge_token "codeberg"
      unset CODEBERG_TOKEN
      The status should be success
      The output should eq "env-token-12345"
    End

    It 'resolves token from .env file'
      echo "CODEBERG_TOKEN=from-dotenv-999" > "$TEST_DIR/.env"
      When call resolve_forge_token "codeberg"
      rm -f "$TEST_DIR/.env"
      The status should be success
      The output should eq "from-dotenv-999"
    End

    It 'resolves token from exported .env line'
      echo "export GITHUB_TOKEN=from-export-dotenv" > "$TEST_DIR/.env"
      When call resolve_forge_token "github"
      rm -f "$TEST_DIR/.env"
      The status should be success
      The output should eq "from-export-dotenv"
    End

    It 'returns failure for unknown forge or missing token'
      unset CODEBERG_TOKEN GITHUB_TOKEN GITLAB_TOKEN SOURCEHUT_TOKEN TILDEGIT_TOKEN || true
      When call resolve_forge_token "unknown-forge"
      The status should be failure
    End
  End

  Describe 'strict zero-awk compliance'
    It 'verifies zero executable awk invocations in src/utils/repo.sh'
      When run grep -E "(^|[[:space:]|;&(])awk([[:space:]|;&)]|$)" "${SHELLSPEC_PROJECT_ROOT}/src/utils/repo.sh"
      The status should be failure
    End
  End

End

# Include the test helper
. "$SHELLSPEC_PROJECT_ROOT/spec/spec_helper.sh"

Describe 'catalog'
  Include "${CCCP_BUNDLE:-$SHELLSPEC_PROJECT_ROOT/src/utils/help.sh}"
  Include "${CCCP_BUNDLE:-$SHELLSPEC_PROJECT_ROOT/src/utils/config_manager.sh}"
  Include "${CCCP_BUNDLE:-$SHELLSPEC_PROJECT_ROOT/src/utils/catalog.sh}"

  setup() {
    TEST_WORKSPACE="$(mktemp -d)"
  }

  cleanup() {
    rm -rf "$TEST_WORKSPACE"
  }

  BeforeEach 'setup'
  AfterEach 'cleanup'

  Describe '_catalog_detect_forge_and_slug'
    It 'detects Codeberg HTTPS URL'
      When call _catalog_detect_forge_and_slug "https://codeberg.org/lumenpink/cccp.git"
      The status should be success
      The output should eq "codeberg lumenpink/cccp false"
    End

    It 'detects Codeberg SSH URL'
      When call _catalog_detect_forge_and_slug "git@codeberg.org:lumenpink/cccp.git"
      The status should be success
      The output should eq "codeberg lumenpink/cccp false"
    End

    It 'detects TildeGit HTTPS URL'
      When call _catalog_detect_forge_and_slug "https://tildegit.org/lumen/cartoes.git"
      The status should be success
      The output should eq "tildegit lumen/cartoes false"
    End

    It 'detects GitHub HTTPS URL'
      When call _catalog_detect_forge_and_slug "https://github.com/KokoroSim/KokoroSim.git"
      The status should be success
      The output should eq "github KokoroSim/KokoroSim false"
    End

    It 'handles unknown or local URLs gracefully'
      When call _catalog_detect_forge_and_slug "/var/repos/myrepo"
      The status should be success
      The output should eq "local unknown false"
    End
  End

  Describe 'heuristic metadata extraction'
    It 'extracts description from .git/description when customized'
      repo_dir="$TEST_WORKSPACE/my-custom-repo"
      mkdir -p "$repo_dir/.git"
      printf 'Custom Git description\n' > "$repo_dir/.git/description"

      When call _catalog_extract_description "$repo_dir"
      The status should be success
      The output should eq "Custom Git description"
    End

    It 'ignores default Git placeholder in .git/description and falls back to Cargo.toml'
      repo_dir="$TEST_WORKSPACE/rust-project"
      mkdir -p "$repo_dir/.git"
      printf "Unnamed repository; edit this file 'description' to name the repository.\n" > "$repo_dir/.git/description"
      cat > "$repo_dir/Cargo.toml" <<'EOF'
[package]
name = "rust-project"
version = "0.1.0"
description = "Fast and safe Rust library"
license = "MIT OR Apache-2.0"
keywords = ["rust", "safety"]
EOF

      When call _catalog_extract_description "$repo_dir"
      The status should be success
      The output should eq "Fast and safe Rust library"
    End

    It 'extracts description and license from package.json'
      repo_dir="$TEST_WORKSPACE/js-project"
      mkdir -p "$repo_dir/.git"
      cat > "$repo_dir/package.json" <<'EOF'
{
  "name": "js-project",
  "description": "Modern frontend widget",
  "license": "Apache-2.0",
  "homepage": "https://widget.example.com"
}
EOF

      When call _catalog_extract_description "$repo_dir"
      The status should be success
      The output should eq "Modern frontend widget"
    End

    It 'extracts license from LICENSE file'
      repo_dir="$TEST_WORKSPACE/lic-project"
      mkdir -p "$repo_dir"
      printf 'MIT License\nCopyright 2026\n' > "$repo_dir/LICENSE"

      When call _catalog_extract_license "$repo_dir"
      The status should be success
      The output should eq "MIT"
    End

    It 'extracts website from package.json'
      repo_dir="$TEST_WORKSPACE/web-project"
      mkdir -p "$repo_dir"
      cat > "$repo_dir/package.json" <<'EOF'
{
  "name": "web-project",
  "homepage": "https://super.example.com"
}
EOF
      When call _catalog_extract_website "$repo_dir"
      The status should be success
      The output should eq "https://super.example.com"
    End
  End

  Describe '_catalog_sync_local'
    It 'scans repos, populates .git/description, and generates catalog.json and README.md'
      repo1="$TEST_WORKSPACE/alpha"
      repo2="$TEST_WORKSPACE/beta"
      mkdir -p "$repo1/.git"
      mkdir -p "$repo2/.git"

      cat > "$repo1/Cargo.toml" <<'EOF'
[package]
name = "alpha"
description = "Alpha Rust project"
license = "MIT"
keywords = ["alpha", "tool"]
EOF
      (cd "$repo1" && git init -q && git config remote.origin.url "https://codeberg.org/lumenpink/alpha.git")

      cat > "$repo2/package.json" <<'EOF'
{
  "name": "beta",
  "description": "Beta Node module",
  "license": "CC0-1.0",
  "keywords": ["beta", "node"]
}
EOF
      (cd "$repo2" && git init -q && git config remote.origin.url "https://tildegit.org/lumen/beta.git")

      When call _catalog_sync_local "$TEST_WORKSPACE"
      The status should be success
      The output should include "[OK] Gravado"
      The path "$TEST_WORKSPACE/catalog.json" should be exist
      The path "$TEST_WORKSPACE/README.md" should be exist
      The contents of file "$repo1/.git/description" should eq "Alpha Rust project"
      The contents of file "$repo2/.git/description" should eq "Beta Node module"
      The contents of file "$TEST_WORKSPACE/catalog.json" should include '"name": "alpha"'
      The contents of file "$TEST_WORKSPACE/catalog.json" should include '"forge": "codeberg"'
      The contents of file "$TEST_WORKSPACE/catalog.json" should include '"name": "beta"'
      The contents of file "$TEST_WORKSPACE/catalog.json" should include '"forge": "tildegit"'
      The contents of file "$TEST_WORKSPACE/README.md" should include '| **[alpha](./alpha)** | 🏔️ Codeberg | `MIT` | Alpha Rust project |'
      The contents of file "$TEST_WORKSPACE/README.md" should include '| **[beta](./beta)** | 🟣 TildeGit | `CC0-1.0` | Beta Node module |'
    End

    It 'fails when directory does not exist'
      When call _catalog_sync_local "$TEST_WORKSPACE/nonexistent"
      The status should be failure
      The error should include "does not exist"
    End
  End

  Describe '_catalog_list'
    It 'outputs formatted text table from existing catalog.json'
      cat > "$TEST_WORKSPACE/catalog.json" <<'EOF'
[
  {
    "name": "superapp",
    "forge": "codeberg",
    "slug": "lumen/superapp",
    "description": "Super web application",
    "topics": ["web", "app"],
    "license": "MIT",
    "website": "https://super.example.com",
    "is_upstream": false
  }
]
EOF
      When call _catalog_list "$TEST_WORKSPACE" 0
      The status should be success
      The output should include "PROJETO"
      The output should include "superapp"
      The output should include "codeberg"
      The output should include "Super web application"
    End

    It 'outputs raw JSON when --json flag is used'
      cat > "$TEST_WORKSPACE/catalog.json" <<'EOF'
[
  {
    "name": "microtool",
    "forge": "github",
    "slug": "user/microtool",
    "description": "Tiny helper",
    "topics": ["cli"],
    "license": "Unlicense",
    "website": "",
    "is_upstream": false
  }
]
EOF
      When call _catalog_list "$TEST_WORKSPACE" 1
      The status should be success
      The output should include '"name": "microtool"'
      The output should include '"slug": "user/microtool"'
    End
  End

  Describe 'cmd_catalog CLI dispatcher'
    It 'displays help on --help'
      When call cmd_catalog --help
      The status should be success
      The output should include "cccp.sh catalog"
      The output should include "Subcommands:"
    End

    It 'rejects unknown option'
      When call cmd_catalog --invalid-flag
      The status should be failure
      The error should include "Unknown catalog subcommand or option"
    End

    It 'executes sync-local via cmd_catalog local'
      repo="$TEST_WORKSPACE/gamerepo"
      mkdir -p "$repo/.git"
      printf 'Video game simulator\n' > "$repo/.git/description"
      (cd "$repo" && git init -q && git config remote.origin.url "https://github.com/myorg/gamerepo.git")

      When call cmd_catalog local --dir "$TEST_WORKSPACE"
      The status should be success
      The output should include "[OK] Gravado"
      The path "$TEST_WORKSPACE/catalog.json" should be exist
      The contents of file "$TEST_WORKSPACE/catalog.json" should include '"name": "gamerepo"'
    End
  End

  Describe '_catalog_sync_remote'
    It 'skips upstream repositories without performing network calls'
      cat > "$TEST_WORKSPACE/catalog.json" <<'EOF'
[
  {
    "name": "upstream-fork",
    "forge": "github",
    "slug": "original-owner/upstream-fork",
    "description": "Third-party fork",
    "topics": ["upstream"],
    "license": "MIT",
    "website": "",
    "is_upstream": true
  }
]
EOF
      When call _catalog_sync_remote "$TEST_WORKSPACE"
      The status should be success
      The output should include "Repositório upstream de terceiros"
      The output should include "Pulando metadados remotos"
    End
  End

  Describe '_catalog_resolve_target_dir'
    setup_env() {
      TEST_DIR="$(mktemp -d)"
      ORIG_HOME="$HOME"
      ORIG_XDG_CONFIG="${XDG_CONFIG_HOME:-}"
      ORIG_XDG_DATA="${XDG_DATA_HOME:-}"
      ORIG_XDG_CACHE="${XDG_CACHE_HOME:-}"
      export HOME="$TEST_DIR"
      unset XDG_CONFIG_HOME || true
      unset XDG_DATA_HOME || true
      unset XDG_CACHE_HOME || true
      unset CATALOG_DIR || true
      unset CCCP_CATALOG_DIR || true
    }

    cleanup_env() {
      export HOME="$ORIG_HOME"
      if [ -n "$ORIG_XDG_CONFIG" ]; then export XDG_CONFIG_HOME="$ORIG_XDG_CONFIG"; else unset XDG_CONFIG_HOME || true; fi
      if [ -n "$ORIG_XDG_DATA" ]; then export XDG_DATA_HOME="$ORIG_XDG_DATA"; else unset XDG_DATA_HOME || true; fi
      if [ -n "$ORIG_XDG_CACHE" ]; then export XDG_CACHE_HOME="$ORIG_XDG_CACHE"; else unset XDG_CACHE_HOME || true; fi
      unset CATALOG_DIR || true
      unset CCCP_CATALOG_DIR || true
      rm -rf "$TEST_DIR"
    }

    BeforeEach 'setup_env'
    AfterEach 'cleanup_env'

    It 'resolves target directory from CLI flag'
      When call _catalog_resolve_target_dir "/custom/cli/path"
      The status should be success
      The output should eq "/custom/cli/path"
    End

    It 'resolves target directory from CATALOG_DIR env var'
      export CATALOG_DIR="/custom/env/path"
      When call _catalog_resolve_target_dir ""
      The status should be success
      The output should eq "/custom/env/path"
    End

    It 'resolves target directory from CCCP_CATALOG_DIR env var'
      export CCCP_CATALOG_DIR="/custom/cccp_env/path"
      When call _catalog_resolve_target_dir ""
      The status should be success
      The output should eq "/custom/cccp_env/path"
    End

    It 'prioritizes CLI flag over CATALOG_DIR env var'
      export CATALOG_DIR="/from/env"
      When call _catalog_resolve_target_dir "/from/flag"
      The status should be success
      The output should eq "/from/flag"
    End

    It 'resolves target directory from global config file'
      cfg_dir="$TEST_DIR/.config/cccp"
      mkdir -p "$cfg_dir"
      echo "catalog_dir = /from/global/config" > "$cfg_dir/config"
      When call _catalog_resolve_target_dir ""
      The status should be success
      The output should eq "/from/global/config"
    End

    It 'resolves target directory from local .cccprc'
      local_repo="$(mktemp -d)"
      (cd "$local_repo" && git init -q && echo "catalog_dir = /from/local/cccprc" > .cccprc)
      test_local_dir() {
        cd "$local_repo"
        _catalog_resolve_target_dir ""
      }
      When call test_local_dir
      The status should be success
      The output should eq "/from/local/cccprc"
      rm -rf "$local_repo"
    End

    It 'prioritizes environment variable over global config'
      cfg_dir="$TEST_DIR/.config/cccp"
      mkdir -p "$cfg_dir"
      echo "catalog_dir = /from/config" > "$cfg_dir/config"
      export CATALOG_DIR="/from/env"
      When call _catalog_resolve_target_dir ""
      The status should be success
      The output should eq "/from/env"
    End

    It 'falls back to ~/src when no flag, env, or config is set'
      When call _catalog_resolve_target_dir ""
      The status should be success
      The output should eq "$TEST_DIR/src"
    End

    It 'expands leading tilde in target directory'
      When call _catalog_resolve_target_dir "~/myprojects"
      The status should be success
      The output should eq "$TEST_DIR/myprojects"
    End
  End

  Describe 'strict zero-awk compliance'
    It 'verifies zero executable awk invocations in src/utils/catalog.sh'
      test_awk() {
        grep -rnw "awk" "$SHELLSPEC_PROJECT_ROOT/src/utils/catalog.sh" | grep -v '#.*awk' || true
      }
      When call test_awk
      The status should be success
      The output should eq ""
    End
  End
End

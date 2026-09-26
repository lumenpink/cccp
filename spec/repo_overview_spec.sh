# Include the test helper
. "$SHELLSPEC_PROJECT_ROOT/spec/spec_helper.sh"

Describe 'Multi-Repository Overview Engine'
  Include "${CCCP_BUNDLE:-$SHELLSPEC_PROJECT_ROOT/src/utils/repo.sh}"
  Include "${CCCP_BUNDLE:-$SHELLSPEC_PROJECT_ROOT/src/utils/help.sh}"

  setup() {
    TEST_WORKSPACE="$(mktemp -d)"
    export NO_COLOR="1"
    
    # Create mock repos
    REPO_CLEAN="$TEST_WORKSPACE/repo_clean"
    REPO_DIRTY="$TEST_WORKSPACE/repo_dirty"
    REPO_ISSUES="$TEST_WORKSPACE/repo_issues"
    REPO_UNPUSHED="$TEST_WORKSPACE/repo_unpushed"

    mkdir -p "$REPO_CLEAN" "$REPO_DIRTY" "$REPO_ISSUES" "$REPO_UNPUSHED"

    # Repo clean
    (
      cd "$REPO_CLEAN"
      git init -q
      git config user.name "Soviet Tester"
      git config user.email "tester@gosplan.su"
      echo "clean" > file.txt
      git add file.txt
      git commit -q -m "feat: initial commit"
    )

    # Repo dirty
    (
      cd "$REPO_DIRTY"
      git init -q
      git config user.name "Soviet Tester"
      git config user.email "tester@gosplan.su"
      echo "initial" > file.txt
      git add file.txt
      git commit -q -m "feat: initial commit"
      echo "unstaged modification" >> file.txt
    )

    # Repo issues
    (
      cd "$REPO_ISSUES"
      git init -q
      git config user.name "Soviet Tester"
      git config user.email "tester@gosplan.su"
      echo "issues" > file.txt
      git add file.txt
      git commit -q -m "feat: initial commit"
      mkdir -p .antigravity
      cat > .antigravity/tickets.md << 'EOF'
# Tickets
## [#101] feat: revolutionary feature
- **Status**: `open`
## [#102] fix: bug in distribution
- **Status**: `open`
EOF
      git add .antigravity/tickets.md
      git commit -q -m "chore: sync tickets"
    )

    # Repo unpushed
    (
      cd "$REPO_UNPUSHED"
      git init -q
      git config user.name "Soviet Tester"
      git config user.email "tester@gosplan.su"
      echo "unpushed" > file.txt
      git add file.txt
      git commit -q -m "feat: initial commit"
      git remote add origin https://codeberg.org/soviet/repo_unpushed.git
    )
  }

  cleanup() {
    rm -rf "$TEST_WORKSPACE"
    unset NO_COLOR
  }

  BeforeEach 'setup'
  AfterEach 'cleanup'

  Describe '_repo_discover_list'
    It 'discovers all repositories in explicit directory'
      When call _repo_discover_list "$TEST_WORKSPACE" ""
      The status should be success
      The output should include "$REPO_CLEAN"
      The output should include "$REPO_DIRTY"
      The output should include "$REPO_ISSUES"
    End

    It 'discovers single repository when directory points to a git repo'
      When call _repo_discover_list "$REPO_CLEAN" ""
      The status should be success
      The output should eq "$REPO_CLEAN"
    End

    It 'discovers repositories defined in catalog.json'
      _test_cat="$TEST_WORKSPACE/catalog.json"
      cat > "$_test_cat" << EOF
[
  {
    "name": "repo_clean",
    "forge": "codeberg"
  },
  {
    "name": "repo_dirty",
    "forge": "codeberg"
  }
]
EOF
      When call _repo_discover_list "" "$_test_cat"
      The status should be success
      The output should include "$REPO_CLEAN"
      The output should include "$REPO_DIRTY"
      The output should not include "$REPO_ISSUES"
    End
  End

  Describe '_repo_audit_single'
    It 'correctly audits a clean repository'
      When call _repo_audit_single "$REPO_CLEAN" 0
      The status should be success
      The output should include "repo_clean"
      # dirty count 0
      The output should include "0"
    End

    It 'correctly identifies dirty status in a repository'
      _audit_dirty() {
        _res=$(_repo_audit_single "$REPO_DIRTY" 0)
        _dirty_cnt=$(printf '%s' "$_res" | cut -f3)
        echo "$_dirty_cnt"
      }
      When call _audit_dirty
      The status should be success
      The output should eq "1"
    End

    It 'correctly identifies open tickets from .antigravity/tickets.md'
      _audit_issues() {
        _res=$(_repo_audit_single "$REPO_ISSUES" 0)
        _issues_cnt=$(printf '%s' "$_res" | cut -f5)
        echo "$_issues_cnt"
      }
      When call _audit_issues
      The status should be success
      The output should eq "2"
    End
  End

  Describe 'cmd_repo'
    It 'displays Gosplan header and summary for all repos'
      When call cmd_repo -d "$TEST_WORKSPACE"
      The status should be success
      The output should include "ГОСПЛАН — REGISTRO GERAL E STATUS DE COOPERATIVAS (REPOS)"
      The output should include "repo_clean"
      The output should include "repo_dirty"
      The output should include "repo_issues"
      The output should include "repo_unpushed"
      The output should include "Resumo do Plano Geral:"
      The output should include "Repositórios auditados: 4"
    End

    It 'filters only dirty repositories when --dirty is specified'
      When call cmd_repo -d "$TEST_WORKSPACE" --dirty
      The status should be success
      The output should include "Filtro ativo: apenas modificados (dirty)"
      The output should include "repo_dirty"
      The output should not include "repo_clean"
      The output should not include "repo_issues"
      The output should not include "repo_unpushed"
    End

    It 'filters only unpushed repositories when --unpushed is specified'
      When call cmd_repo -d "$TEST_WORKSPACE" --unpushed
      The status should be success
      The output should include "Filtro ativo: apenas pendentes de envio (unpushed)"
      The output should include "repo_unpushed"
      The output should not include "repo_clean"
      The output should not include "repo_dirty"
      The output should not include "repo_issues"
    End

    It 'filters only repositories with open issues when --issues is specified'
      When call cmd_repo -d "$TEST_WORKSPACE" --issues
      The status should be success
      The output should include "Filtro ativo: apenas com tickets em aberto (issues)"
      The output should include "repo_issues"
      The output should not include "repo_clean"
      The output should not include "repo_dirty"
      The output should not include "repo_unpushed"
    End

    It 'outputs valid structured JSON when --json is passed'
      When call cmd_repo -d "$TEST_WORKSPACE" --json
      The status should be success
      The output should include '"name": "repo_clean"'
      The output should include '"name": "repo_dirty"'
      The output should include '"name": "repo_issues"'
      The output should include '"dirty": 1'
      The output should include '"issues": 2'
    End

    It 'displays help when -h is passed'
      When call cmd_repo -h
      The status should be success
      The output should include "cccp.sh repo - Gosplan Multi-Repository Management & Overview"
    End
  End
End

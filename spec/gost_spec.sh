# Include the test helper
. "$SHELLSPEC_PROJECT_ROOT/spec/spec_helper.sh"

Describe 'GOST Compliance Engine'
  Include "${CCCP_BUNDLE:-$SHELLSPEC_PROJECT_ROOT/src/utils/gost.sh}"

  setup() {
    TEST_WORKSPACE="$(mktemp -d)"
    MOCK_GOST_DIR="$(mktemp -d)"

    # Setup a mock upstream gost repository structure for tests
    mkdir -p "$MOCK_GOST_DIR/profiles" "$MOCK_GOST_DIR/specs/languages" "$MOCK_GOST_DIR/upgrades"
    echo "# Mock GOST Spec" > "$MOCK_GOST_DIR/SPEC.md"
    echo "# Mock Lumen Profile" > "$MOCK_GOST_DIR/profiles/lumen.md"
    echo '{"rules":[]}' > "$MOCK_GOST_DIR/specs/rules.json"
    echo "# Shell Spec" > "$MOCK_GOST_DIR/specs/languages/shell.md"
    echo "# JS Spec" > "$MOCK_GOST_DIR/specs/languages/javascript.md"
    echo "# Python Spec" > "$MOCK_GOST_DIR/specs/languages/python.md"
    echo "# Migration 1.0 to 1.1" > "$MOCK_GOST_DIR/upgrades/from-1.0-to-1.1.md"

    export GOST_LOCAL_PATH="$MOCK_GOST_DIR"
    export NO_COLOR="1"
  }

  cleanup() {
    rm -rf "$TEST_WORKSPACE" "$MOCK_GOST_DIR"
    unset GOST_LOCAL_PATH
    unset NO_COLOR
  }

  BeforeEach 'setup'
  AfterEach 'cleanup'

  Describe '_gost_read_gostver'
    It 'returns failure when GOSTVER is not found'
      When call _gost_read_gostver "$TEST_WORKSPACE"
      The status should be failure
    End

    It 'parses GOST_VERSION, GOST_PROFILE_URL, and GOST_LANGUAGES correctly'
      cat > "$TEST_WORKSPACE/GOSTVER" << 'EOF'
# GOSTVER declaration
GOST_VERSION="1.0.0-draft"
GOST_PROFILE_URL="profiles/lumen.md"
GOST_LANGUAGES="shell,javascript"
EOF
      When call _gost_read_gostver "$TEST_WORKSPACE"
      The status should be success
      The variable GOST_VERSION should eq "1.0.0-draft"
      The variable GOST_PROFILE_URL should eq "profiles/lumen.md"
      The variable GOST_LANGUAGES should eq "shell,javascript"
    End
  End

  Describe '_gost_is_semver'
    It 'accepts standard SemVer 2.0.0 versions'
      _test_semver() {
        _gost_is_semver "1.0.0" && \
        _gost_is_semver "2.4.0-dev.1" && \
        _gost_is_semver "0.1.0+build.123" && \
        _gost_is_semver "10.20.30-rc.1+20260925"
      }
      When call _test_semver
      The status should be success
    End

    It 'rejects non-SemVer version formats'
      _test_invalid() {
        ! _gost_is_semver "1.0" && \
        ! _gost_is_semver "v1.0.0" && \
        ! _gost_is_semver "invalid-ver"
      }
      When call _test_invalid
      The status should be success
    End
  End

  Describe 'cmd_gost_sync'
    It 'fails when GOSTVER is missing'
      When call cmd_gost_sync "$TEST_WORKSPACE"
      The status should be failure
      The error should include "Erro: Arquivo GOSTVER não encontrado"
    End

    It 'synchronizes GOST.md, PROFILE.md, rules.json and selective languages'
      cat > "$TEST_WORKSPACE/GOSTVER" << 'EOF'
GOST_VERSION="1.0.0"
GOST_PROFILE_URL="profiles/lumen.md"
GOST_LANGUAGES="shell,javascript"
EOF
      When call cmd_gost_sync "$TEST_WORKSPACE"
      The status should be success
      The output should include "Sincronização GOST v1.0.0 concluída com sucesso"
      The path "$TEST_WORKSPACE/.gost/GOST.md" should be exist
      The path "$TEST_WORKSPACE/.gost/PROFILE.md" should be exist
      The path "$TEST_WORKSPACE/.gost/rules.json" should be exist
      The path "$TEST_WORKSPACE/.gost/languages/shell.md" should be exist
      The path "$TEST_WORKSPACE/.gost/languages/javascript.md" should be exist
      The path "$TEST_WORKSPACE/.gost/languages/python.md" should not be exist
    End
  End

  Describe 'cmd_gost_check and cmd_check'
    It 'reports failure when core files are missing'
      cat > "$TEST_WORKSPACE/GOSTVER" << 'EOF'
GOST_VERSION="1.0.0"
EOF
      When call cmd_gost_check "$TEST_WORKSPACE"
      The status should be failure
      The output should include "[FAIL] GOST-01-LICENSE"
      The error should include "Inconformidades críticas detectadas"
    End

    It 'passes when all MUST requirements are met'
      cat > "$TEST_WORKSPACE/GOSTVER" << 'EOF'
GOST_VERSION="1.0.0"
GOST_PROFILE_URL="profiles/lumen.md"
GOST_LANGUAGES="shell"
EOF
      echo "GPL-3.0" > "$TEST_WORKSPACE/LICENSE"
      echo "# Readme" > "$TEST_WORKSPACE/README.md"
      echo "1.0.0" > "$TEST_WORKSPACE/VERSION"
      echo "# Changelog" > "$TEST_WORKSPACE/CHANGELOG.md"
      echo "*.log" > "$TEST_WORKSPACE/.gitignore"
      echo "root = true" > "$TEST_WORKSPACE/.editorconfig"
      mkdir -p "$TEST_WORKSPACE/.gost/languages"
      echo "# Shell Spec" > "$TEST_WORKSPACE/.gost/languages/shell.md"

      # Initialize dummy git repository
      (
        cd "$TEST_WORKSPACE"
        git init -q
        git config user.name "Tester"
        git config user.email "test@example.com"
        git checkout -b feat/42-gost-check -q 2>/dev/null || git checkout -b feat/42-gost-check -q
        git add .
        git commit -m "feat(check): initial commit (closes #42)" -q
      )

      When call cmd_check "$TEST_WORKSPACE"
      The status should be success
      The output should include "[PASS] GOST-16-GOSTVER"
      The output should include "[PASS] GOST-01-LICENSE"
      The output should include "[PASS] GOST-02-ANATOMY-ROOT"
      The output should include "[PASS] GOST-03-VERSION-SEMVER"
      The output should include "[PASS] GOST-04-TICKETS-COMMITS"
      The output should include "[PASS] GOST-05-BRANCH-TICKET"
      The output should include "Projeto aprovado pela Comissão de Normas Técnicas GOST."
    End
  End

  Describe 'cmd_gost_upgrade'
    It 'requires a target version argument'
      When call cmd_gost_upgrade "" "$TEST_WORKSPACE"
      The status should be failure
      The error should include "Erro: Versão alvo não informada."
    End

    It 'displays migration guide and upgrades GOSTVER'
      cat > "$TEST_WORKSPACE/GOSTVER" << 'EOF'
GOST_VERSION="1.0.0"
GOST_PROFILE_URL="profiles/lumen.md"
GOST_LANGUAGES="shell"
EOF
      When call cmd_gost_upgrade "1.1.0" "$TEST_WORKSPACE"
      The status should be success
      The output should include "GUIA DE MIGRAÇÃO"
      The output should include "GOSTVER atualizado para GOST_VERSION=\"1.1.0\""
      The contents of file "$TEST_WORKSPACE/GOSTVER" should include 'GOST_VERSION="1.1.0"'
    End
  End

  Describe 'help outputs'
    It 'displays check help'
      When call cmd_check --help
      The status should be success
      The output should include "Uso: cccp check"
      The output should include "--strict"
    End

    It 'displays gost help'
      When call cmd_gost help
      The status should be success
      The output should include "Uso: cccp gost"
      The output should include "sync"
      The output should include "check"
      The output should include "upgrade"
    End
  End
End

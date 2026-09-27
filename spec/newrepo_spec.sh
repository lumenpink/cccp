# Include the test helper
. "$SHELLSPEC_PROJECT_ROOT/spec/spec_helper.sh"

Describe 'newrepo - GOST Scaffolding Engine'
  Include "${CCCP_BUNDLE:-$SHELLSPEC_PROJECT_ROOT/src/utils/newrepo.sh}"
  Include "${CCCP_BUNDLE:-$SHELLSPEC_PROJECT_ROOT/src/utils/gost.sh}"
  Include "${CCCP_BUNDLE:-$SHELLSPEC_PROJECT_ROOT/src/utils/repo.sh}"

  setup() {
    TEST_WORKSPACE="$(mktemp -d)"
    export NO_COLOR="1"
  }

  cleanup() {
    rm -rf "$TEST_WORKSPACE"
    unset NO_COLOR
  }

  BeforeEach 'setup'
  AfterEach 'cleanup'

  Describe 'help and usage'
    It 'displays help message on -h'
      When call cmd_newrepo -h
      The status should be success
      The output should include "cccp newrepo - Scaffolding Automatizado de Repositórios no Modelo GOST"
      The output should include "Uso:"
      The output should include "--license"
      The output should include "--lang"
      The output should include "--no-git"
    End

    It 'displays help message on --help'
      When call cmd_newrepo --help
      The status should be success
      The output should include "cccp newrepo - Scaffolding Automatizado de Repositórios no Modelo GOST"
    End
  End

  Describe 'argument validation'
    It 'fails when neither name nor root is provided'
      When call cmd_newrepo
      The status should be failure
      The error should include "Erro: Informe o nome do repositório a ser criado"
    End

    It 'fails on unknown options'
      When call cmd_newrepo meu-app --invalid-option
      The status should be failure
      The error should include "Erro: Opção desconhecida '--invalid-option'"
    End

    It 'fails when --root is passed without argument'
      When call cmd_newrepo --root
      The status should be failure
      The error should include "Erro: --root requer um diretório"
    End
  End

  Describe 'scaffolding creation'
    It 'creates a fully compliant repository tree by default'
      _run_newrepo() {
        cd "$TEST_WORKSPACE"
        cmd_newrepo meu-projeto --no-sync
      }
      When call _run_newrepo
      The status should be success
      The output should include "Scaffolding de Repositório: meu-projeto"
      The output should include "✔ Repositório 'meu-projeto' criado com sucesso"

      The path "$TEST_WORKSPACE/meu-projeto/GOSTVER" should be exist
      The path "$TEST_WORKSPACE/meu-projeto/.editorconfig" should be exist
      The path "$TEST_WORKSPACE/meu-projeto/.gitignore" should be exist
      The path "$TEST_WORKSPACE/meu-projeto/VERSION" should be exist
      The path "$TEST_WORKSPACE/meu-projeto/CHANGELOG.md" should be exist
      The path "$TEST_WORKSPACE/meu-projeto/LICENSE" should be exist
      The path "$TEST_WORKSPACE/meu-projeto/README.md" should be exist
      The path "$TEST_WORKSPACE/meu-projeto/site/_headers" should be exist
      The path "$TEST_WORKSPACE/meu-projeto/site/index.html" should be exist
      The path "$TEST_WORKSPACE/meu-projeto/.forgejo/workflows/ci.yaml" should be exist
      The path "$TEST_WORKSPACE/meu-projeto/.git" should be exist

      The contents of file "$TEST_WORKSPACE/meu-projeto/GOSTVER" should include 'GOST_VERSION="1.1.0"'
      The contents of file "$TEST_WORKSPACE/meu-projeto/VERSION" should include '0.1.0-dev.1'
      The contents of file "$TEST_WORKSPACE/meu-projeto/.editorconfig" should include 'indent_size = 2'
      The contents of file "$TEST_WORKSPACE/meu-projeto/site/_headers" should include 'X-Content-Type-Options: nosniff'
    End

    It 'achieves 100% compliance in cccp check audit'
      _run_and_audit() {
        cd "$TEST_WORKSPACE"
        cmd_newrepo servico-gost --no-sync -q
        mkdir -p "$TEST_WORKSPACE/servico-gost/.gost/languages"
        echo "# Shell Spec" > "$TEST_WORKSPACE/servico-gost/.gost/languages/shell.md"
        cmd_gost_check "$TEST_WORKSPACE/servico-gost"
      }
      When call _run_and_audit
      The status should be success
      The output should include "[PASS] GOST-16-GOSTVER"
      The output should include "[PASS] GOST-01-LICENSE"
      The output should include "[PASS] GOST-02-ANATOMY-ROOT"
      The output should include "[PASS] GOST-03-VERSION-SEMVER"
      The output should include "[PASS] GOST-04-TICKETS-COMMITS"
      The output should include "[PASS] GOST-05-BRANCH-TICKET"
      The output should include "[PASS] GOST-09-SITE-HEADERS"
      The output should include "[PASS] GOST-10-CI-SEPARATION"
      The output should include "[PASS] GOST-17-LANGUAGE-SPECS"
      The output should include "Índice de Conformidade Gosplan: 100%"
    End
  End

  Describe 'custom configuration options'
    It 'customizes license, languages, and description'
      _run_custom() {
        cd "$TEST_WORKSPACE"
        cmd_newrepo api-rust \
          --lang "rust,shell" \
          --license "MIT" \
          --desc "Serviço central de telemetria" \
          --no-sync \
          -q
      }
      When call _run_custom
      The status should be success
      The contents of file "$TEST_WORKSPACE/api-rust/GOSTVER" should include 'GOST_LANGUAGES="rust,shell"'
      The contents of file "$TEST_WORKSPACE/api-rust/LICENSE" should include 'MIT License'
      The contents of file "$TEST_WORKSPACE/api-rust/README.md" should include 'Serviço central de telemetria'
      The contents of file "$TEST_WORKSPACE/api-rust/site/index.html" should include 'Serviço central de telemetria'
    End

    It 'supports --no-git flag to avoid git initialization'
      _run_nogit() {
        cd "$TEST_WORKSPACE"
        cmd_newrepo docs-apenas --no-git --no-sync -q
      }
      When call _run_nogit
      The status should be success
      The path "$TEST_WORKSPACE/docs-apenas/.git" should not be exist
      The path "$TEST_WORKSPACE/docs-apenas/GOSTVER" should be exist
    End

    It 'supports --no-ci flag to skip CI workflow creation'
      _run_noci() {
        cd "$TEST_WORKSPACE"
        cmd_newrepo app-sem-ci --no-ci --no-sync -q
      }
      When call _run_noci
      The status should be success
      The path "$TEST_WORKSPACE/app-sem-ci/.forgejo/workflows/ci.yaml" should not be exist
    End
  End

  Describe 'idempotency'
    It 'does not overwrite existing custom files'
      _run_idempotent() {
        mkdir -p "$TEST_WORKSPACE/existente"
        echo "CUSTOM_README_CONTENT" > "$TEST_WORKSPACE/existente/README.md"
        cmd_newrepo --root "$TEST_WORKSPACE/existente" --name "existente" --no-sync
      }
      When call _run_idempotent
      The status should be success
      The output should include "[EXISTE] README.md (preservado)"
      The contents of file "$TEST_WORKSPACE/existente/README.md" should include 'CUSTOM_README_CONTENT'
      The path "$TEST_WORKSPACE/existente/GOSTVER" should be exist
    End
  End

  Describe 'delegation from cmd_repo'
    It 'delegates cccp repo new to cmd_newrepo'
      cmd_newrepo() {
        echo "MOCK_CMD_NEWREPO_CALLED: $@"
        return 0
      }
      When call cmd_repo new meu-servico --lang python
      The status should be success
      The output should include "MOCK_CMD_NEWREPO_CALLED: meu-servico --lang python"
    End
  End

  Describe 'strict zero-awk compliance'
    It 'verifies zero executable awk invocations in src/utils/newrepo.sh'
      test_awk() {
        grep -rnw "awk" "$SHELLSPEC_PROJECT_ROOT/src/utils/newrepo.sh" | grep -v '#.*awk' || true
      }
      When call test_awk
      The status should be success
      The output should eq ""
    End
  End
End

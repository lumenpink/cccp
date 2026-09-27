# Include the test helper
. "$SHELLSPEC_PROJECT_ROOT/spec/spec_helper.sh"

Describe 'CCCP Dependencies Manager (cmd_deps)'
  Include "${CCCP_BUNDLE:-$SHELLSPEC_PROJECT_ROOT/src/utils/deps.sh}"
  Include "${CCCP_BUNDLE:-$SHELLSPEC_PROJECT_ROOT/src/utils/comrades.sh}"
  Include "${CCCP_BUNDLE:-$SHELLSPEC_PROJECT_ROOT/src/utils/help.sh}"

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

  Describe '_deps_version_ge'
    It 'returns success when versions are strictly equal'
      When call _deps_version_ge "2.25.0" "2.25.0"
      The status should be success
    End

    It 'returns success when actual major is greater'
      When call _deps_version_ge "3.0.0" "2.25.0"
      The status should be success
    End

    It 'returns failure when actual major is lower'
      When call _deps_version_ge "1.9.9" "2.25.0"
      The status should be failure
    End

    It 'returns success when actual minor is greater'
      When call _deps_version_ge "2.40.1" "2.25.0"
      The status should be success
    End

    It 'returns failure when actual minor is lower'
      When call _deps_version_ge "2.24.9" "2.25.0"
      The status should be failure
    End

    It 'returns success when actual patch is greater'
      When call _deps_version_ge "2.25.1" "2.25.0"
      The status should be success
    End

    It 'strips leading v prefix transparently'
      When call _deps_version_ge "v2.30.0" "2.25.0"
      The status should be success
    End

    It 'considers development builds as conforming'
      When call _deps_version_ge "development" "0.9.0"
      The status should be success
    End

    It 'considers dev branch tags as conforming'
      When call _deps_version_ge "dev.1" "0.1.0"
      The status should be success
    End
  End

  Describe 'cmd_deps_list'
    It 'outputs Gosplan header and lists mandatory and optional dependencies'
      When call cmd_deps_list
      The status should be success
      The output should include "Catálogo de Dependências"
      The output should include "git"
      The output should include "docker"
      The output should include "act"
      The output should include "sv"
      The output should include "kv"
      The output should include "mk"
      The output should include "curl"
      The output should include "jq"
    End
  End

  Describe 'cmd_deps_check'
    It 'successfully audits existing core dependency git'
      When call cmd_deps_check git
      The status should be success
      The output should include "git"
      The output should include "[OK]"
      The output should include "Auditoria de dependências satisfeita"
    End

    It 'filters check when specific dependency is specified'
      When call cmd_deps_check git
      The status should be success
      The output should include "git"
      The output should not include "docker"
    End
  End

  Describe 'cmd_deps router and help'
    It 'shows help on --help'
      When call cmd_deps --help
      The status should be success
      The output should include "Dependencies & Toolchain Manager"
      The output should include "list"
      The output should include "check"
      The output should include "install"
    End

    It 'dispatches to list on list subcommand'
      When call cmd_deps list
      The status should be success
      The output should include "Catálogo de Dependências"
    End
  End

  Describe 'strict zero-awk compliance'
    It 'verifies zero executable awk invocations in src/utils/deps.sh'
      When run grep -E "(^|[[:space:]|;&(])awk([[:space:]|;&)]|$)" "$SHELLSPEC_PROJECT_ROOT/src/utils/deps.sh"
      The status should be failure
    End
  End

End

# Include the test helper
. "$SHELLSPEC_PROJECT_ROOT/spec/spec_helper.sh"

Describe 'validation'
  Include "${CCCP_BUNDLE:-$SHELLSPEC_PROJECT_ROOT/src/utils/validation.sh}"

  Describe 'validate_commit_message'
    It 'accepts valid commit message with type only'
      When call validate_commit_message "feat: add new feature"
      The status should be success
    End

    It 'accepts valid commit message with scope'
      When call validate_commit_message "feat(ui): add new button"
      The status should be success
    End

    It 'accepts valid commit message with multiple scopes'
      When call validate_commit_message "feat(ui,api): add login feature"
      The status should be success
    End

    It 'accepts valid commit message with subscope'
      When call validate_commit_message "feat(ui/components): add button component"
      The status should be success
    End

    It 'accepts breaking change marker ! in type'
      When call validate_commit_message "feat!: drop legacy endpoint"
      The status should be success
    End

    It 'accepts breaking change marker ! after scope'
      When call validate_commit_message "feat(api)!: drop legacy endpoint"
      The status should be success
    End

    It 'permits unlisted scope by default (STRICT_SCOPES=0)'
      When call validate_commit_message "feat(arbitrary-scope): something"
      The status should be success
    End

    It 'permits unlisted subscope by default (STRICT_SUBSCOPES=0)'
      When call validate_commit_message "feat(ui/arbitrary-sub): something"
      The status should be success
    End

    It 'rejects invalid commit type'
      When call validate_commit_message "invalid: some message"
      The status should be failure
      The output should include "Error: Invalid type 'invalid'"
    End

    It 'rejects empty subject'
      When call validate_commit_message "feat:"
      The status should be failure
      The output should include "Error: Commit message must have a subject"
    End

    It 'rejects invalid scope when STRICT_SCOPES=1'
      BeforeCall 'STRICT_SCOPES=1'
      When call validate_commit_message "feat(invalid): some message"
      The status should be failure
      The output should include "Error: Invalid scope 'invalid'"
    End

    It 'rejects invalid subscope when STRICT_SUBSCOPES=1'
      BeforeCall 'STRICT_SUBSCOPES=1'
      When call validate_commit_message "feat(ui/invalid): some message"
      The status should be failure
      The output should include "Error: Invalid subscope 'invalid'"
    End

    It 'maintains backward compatibility with ALLOW_ANY_SCOPE=0'
      BeforeCall 'ALLOW_ANY_SCOPE=0'
      When call validate_commit_message "feat(invalid): some message"
      The status should be failure
      The output should include "Error: Invalid scope 'invalid'"
    End

    It 'maintains backward compatibility with ALLOW_ANY_SUBSCOPE=0'
      BeforeCall 'ALLOW_ANY_SUBSCOPE=0'
      When call validate_commit_message "feat(ui/invalid): some message"
      The status should be failure
      The output should include "Error: Invalid subscope 'invalid'"
    End

    It 'rejects empty parentheses'
      When call validate_commit_message "feat(): new awesome feature"
      The status should be failure
      The output should include "Error: Scope cannot be empty"
    End

    It 'respects DISABLE_SUBSCOPES flag'
      BeforeCall 'DISABLE_SUBSCOPES=1'
      When call validate_commit_message "feat(ui/components): some message"
      The status should be failure
      The output should include "Error: Subscopes are disabled"
    End

    It 'respects DISABLE_MULTIPLE_SCOPES flag'
      BeforeCall 'DISABLE_MULTIPLE_SCOPES=1'
      When call validate_commit_message "feat(ui,api): some message"
      The status should be failure
      The output should include "Error: Multiple scopes are disabled"
    End

    It 'accepts custom types when configured in COMMIT_TYPES'
      BeforeCall 'COMMIT_TYPES="feat fix decree"'
      When call validate_commit_message "decree: state proclamation"
      The status should be success
    End

    It 'rejects unconfigured types when COMMIT_TYPES is customized'
      BeforeCall 'COMMIT_TYPES="feat fix decree"'
      When call validate_commit_message "docs: write documentation"
      The status should be failure
      The output should include "Error: Invalid type 'docs'"
    End

    It 'permits arbitrary alphanumeric types when STRICT_TYPES=0'
      BeforeCall 'STRICT_TYPES=0'
      When call validate_commit_message "tractor: harvest wheat"
      The status should be success
    End

    It 'rejects invalid characters in type when STRICT_TYPES=0'
      BeforeCall 'STRICT_TYPES=0'
      When call validate_commit_message "bad@type: invalid characters"
      The status should be failure
      The output should include "Type must consist of alphanumeric characters"
    End
  End

  Describe 'check_system_tools'
    It 'succeeds when all standard POSIX tools are present'
      When call check_system_tools
      The status should be success
    End

    It 'fails and reports missing tools when a tool is absent'
      PATH="/dev/null"
      When call check_system_tools
      The status should be failure
      The error should include "Required system tools are missing from PATH"
    End
  End

  Describe 'check_git_repo'
    It 'fails when executed outside a git repository'
      NON_GIT_DIR="$(mktemp -d)"
      cd "$NON_GIT_DIR"
      When call check_git_repo
      The status should be failure
      The error should include "Not a git repository"
      rm -rf "$NON_GIT_DIR"
    End
  End
End
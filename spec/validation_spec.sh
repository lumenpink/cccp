# Include the test helper
. "$SHELLSPEC_PROJECT_ROOT/spec/spec_helper.sh"

Describe 'validation'
  Include "$SHELLSPEC_PROJECT_ROOT/src/utils/validation.sh"

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
  End
End
# Include the test helper
. "$SHELLSPEC_PROJECT_ROOT/spec/spec_helper.sh"

Describe 'soviet'
  Include "${CCCP_BUNDLE:-$SHELLSPEC_PROJECT_ROOT/src/utils/soviet.sh}"
  Include "${CCCP_BUNDLE:-$SHELLSPEC_PROJECT_ROOT/src/main.sh}"

  Describe 'show_soviet'
    It 'displays Soviet ASCII emblem, Order No. 227, and party directives'
      When call show_soviet
      The status should be success
      The output should include "CONVENTIONAL COMMITS COMPLIANCE PROGRAM"
      The output should include "Workers of the World, Conventionalize! ☭"
      The output should include "ORDER NO. 227: Not one step back from Conventional Commits!"
      The output should include "Five-Year Plan for 100% compliant Git history is fulfilling its quota!"
      The output should include "Directives:"
      The output should include "Glory to the Standardized Commit History!"
    End
  End

  Describe 'Soviet command aliases via main dispatcher'
    It 'invokes show_soviet via soviet command'
      When run main "soviet"
      The status should be success
      The output should include "ORDER NO. 227"
    End

    It 'invokes show_soviet via sputnik alias'
      When run main "sputnik"
      The status should be success
      The output should include "ORDER NO. 227"
    End

    It 'invokes show_soviet via anthem alias'
      When run main "anthem"
      The status should be success
      The output should include "ORDER NO. 227"
    End

    It 'invokes show_soviet via gosplan alias'
      When run main "gosplan"
      The status should be success
      The output should include "ORDER NO. 227"
    End
  End
End

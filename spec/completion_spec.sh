# Include the test helper
. "$SHELLSPEC_PROJECT_ROOT/spec/spec_helper.sh"

Describe 'completion'
  Include "$SHELLSPEC_PROJECT_ROOT/src/utils/completion.sh"

  Describe 'cmd_completion'
    It 'generates valid bash completion script'
      When call cmd_completion "bash"
      The status should be success
      The output should include "# CCCP Bash Completion"
      The output should include "_cccp_completions()"
      The output should include "complete -F _cccp_completions cccp cccp.sh"
    End

    It 'generates valid zsh completion script'
      When call cmd_completion "zsh"
      The status should be success
      The output should include "#compdef cccp cccp.sh"
      The output should include "_cccp()"
      The output should include "_arguments"
    End

    It 'generates valid fish completion script'
      When call cmd_completion "fish"
      The status should be success
      The output should include "# CCCP Fish Completion"
      The output should include "complete -c cccp -f"
      The output should include "complete -c cccp -n"
    End

    It 'rejects unsupported shell with clear error'
      When call cmd_completion "powershell"
      The status should be failure
      The error should include "Error: Unsupported shell 'powershell'"
      The error should include "Supported shells: bash, zsh, fish"
    End

    It 'defaults to bash when no argument is specified and SHELL is unset'
      run_default() {
        SHELL="" cmd_completion
      }
      When call run_default
      The status should be success
      The output should include "# CCCP Bash Completion"
    End
  End
End

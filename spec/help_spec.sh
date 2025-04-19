#!/bin/sh

Describe 'Help Function'
  Include src/utils/help.sh

  Describe 'show_help'
    It 'displays help information'
      When call show_help
      The output should include "Git Conventional Commits Helper Script"
      The output should include "Usage:"
      The output should include "Commands:"
      The output should include "commit <message>"
      The output should include "install"
      The output should include "version"
      The output should include "changelog"
      The output should include "help"
      The output should include "Commit Message Format:"
      The output should include "Types:"
      The output should include "Scopes:"
      The output should include "Subscopes:"
      The output should include "Environment Variables:"
      The output should include "Examples:"
    End
  End
End 
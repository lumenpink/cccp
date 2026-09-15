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

    It 'displays help information for version command'
      When call show_help "version"
      The output should include "Predictive Semantic Versioning"
      The output should include "Baseline Discovery"
      The output should include "Commit Inspection"
      The output should include "MAJOR bump"
      The output should include "MINOR bump"
      The output should include "PATCH bump"
    End

    It 'displays help information for tag command'
      When call show_help "tag"
      The output should include "Create Release Tag"
      The output should include "--no-v"
      The output should include "-m, --message"
    End

    It 'displays help information for config command'
      When call show_help "config"
      The output should include "Manage Hierarchical Configuration"
      The output should include "--global"
      The output should include "--local"
      The output should include "--unset"
    End
  End
End 
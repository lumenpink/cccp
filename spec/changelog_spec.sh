#!/bin/sh

# Include the test helper
. "$SHELLSPEC_PROJECT_ROOT/spec/spec_helper.sh"

Describe 'changelog'
  Include "${CCCP_BUNDLE:-$SHELLSPEC_PROJECT_ROOT/src/utils/changelog.sh}"

  setup() {
    # Create a temporary directory for the test repository
    TEST_DIR="$(mktemp -d)"
    cd "$TEST_DIR"
    
    # Initialize git repository
    git init
    git config user.email "test@example.com"
    git config user.name "Test User"
    
    # Create initial commit
    touch README.md
    git add README.md
    git commit -m "initial commit"
    
    # Create a tag
    git tag -a "v1.0.0" -m "Version 1.0.0"
  }

  cleanup() {
    # Clean up the temporary directory
    rm -rf "$TEST_DIR"
  }

  BeforeAll 'setup'
  AfterAll 'cleanup'

  Describe 'generate_changelog'
    add_commit() {
      echo "test" >> test.txt
      git add test.txt
      git commit -m "$1"
    }

    BeforeEach 'rm -f CHANGELOG.md'

    It 'creates CHANGELOG.md file'
      When call generate_changelog
      The file CHANGELOG.md should be exist
    End

    It 'includes Unreleased section'
      When call generate_changelog
      The contents of file CHANGELOG.md should include "[Unreleased]"
    End

    It 'includes Previous Releases section'
      When call generate_changelog
      The contents of file CHANGELOG.md should include "Previous Releases"
    End

    It 'groups commits by type'
      add_commit "feat: new feature"
      add_commit "fix: bug fix"
      add_commit "perf: performance improvement"
      When call generate_changelog
      The contents of file CHANGELOG.md should include "Features"
      The contents of file CHANGELOG.md should include "Bug Fixes"
      The contents of file CHANGELOG.md should include "Performance Improvements"
    End

    It 'only includes commits from CHANGELOG_TYPES'
      add_commit "feat: new feature"
      add_commit "chore: update dependencies"
      When call generate_changelog
      The contents of file CHANGELOG.md should include "Features"
      The contents of file CHANGELOG.md should not include "chore"
    End

    It 'includes commits with scopes'
      add_commit "feat(api): add new endpoint"
      add_commit "fix(security): patch vulnerability"
      add_commit "perf(database): optimize queries"
      When call generate_changelog
      The contents of file CHANGELOG.md should include "  - (api) add new endpoint"
      The contents of file CHANGELOG.md should include "  - (security) patch vulnerability"
      The contents of file CHANGELOG.md should include "  - (database) optimize queries"
    End

  End

  Describe 'format_changelog_entries'
    It 'omits section headings when category has no commits'
      When call format_changelog_entries "  - new feature" "" ""
      The output should include "### Features"
      The output should not include "### Bug Fixes"
      The output should not include "### Performance Improvements"
    End

    It 'provides default fallback message when no categorized commits exist'
      When call format_changelog_entries "" "" ""
      The output should include "Routine maintenance, documentation updates, and operational improvements."
      The output should not include "### Features"
      The output should not include "### Bug Fixes"
      The output should not include "### Performance Improvements"
    End
  End

  Describe 'format_commit_message'
    It 'formats commit message without scope'
      When call format_commit_message "feat: revolutionary tractor assembly"
      The output should eq "  - revolutionary tractor assembly"
    End

    It 'formats commit message with single scope'
      When call format_commit_message "feat(engine): high efficiency turbine"
      The output should eq "  - (engine) high efficiency turbine"
    End

    It 'formats commit message with multiple scopes'
      When call format_commit_message "fix(ui,api): synchronization discrepancy"
      The output should eq "  - (ui,api) synchronization discrepancy"
    End

    It 'formats commit message with subscope'
      When call format_commit_message "perf(db/queries): batch collective records"
      The output should eq "  - (db/queries) batch collective records"
    End
  End

  Describe 'format_tag_header'
    It 'formats standard tag header without title'
      git tag -a "v1.1.0" -m "" -f
      When call format_tag_header "v1.1.0"
      The output should eq "### [v1.1.0]"
    End

    It 'formats tag header with Release title prefix'
      git tag -a "v1.2.0" -m "Release v1.2.0: Five-Year Plan Milestone" -f
      When call format_tag_header "v1.2.0"
      The output should eq "### [v1.2.0] - Five-Year Plan Milestone"
    End

    It 'formats tag header with custom subject title'
      git tag -a "v1.3.0" -m "feat: Cosmic Sputnik Launch" -f
      When call format_tag_header "v1.3.0"
      The output should eq "### [v1.3.0] - Cosmic Sputnik Launch"
    End
  End
End 
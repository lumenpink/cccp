# Include the test helper
. "$SHELLSPEC_PROJECT_ROOT/spec/spec_helper.sh"

Describe 'tag'
  Include "$SHELLSPEC_PROJECT_ROOT/src/utils/version.sh"
  Include "$SHELLSPEC_PROJECT_ROOT/src/utils/changelog.sh"
  Include "$SHELLSPEC_PROJECT_ROOT/src/utils/tag.sh"

  setup() {
    TEST_DIR="$(mktemp -d)"
    cd "$TEST_DIR"

    git init
    git config user.email "test@example.com"
    git config user.name "Test User"

    touch README.md
    git add README.md
    git commit -m "initial commit"
  }

  cleanup() {
    rm -rf "$TEST_DIR"
  }

  BeforeAll 'setup'
  AfterAll 'cleanup'

  Describe 'create_tag'
    It 'fails when working directory has uncommitted changes'
      echo "uncommitted change" >> README.md
      When call create_tag 2
      The error should include "Working directory has uncommitted changes"
      The status should be failure
      # Revert uncommitted change for subsequent tests
      git checkout -- README.md
    End

    It 'creates tag with major version number and release commit'
      When call create_tag 2
      The output should include "Release commit created: chore(release): v2.0.0"
      The output should include "Tag 'v2.0.0' created successfully."
      The contents of file VERSION should eq "2.0.0"
      The file CHANGELOG.md should be exist
    End

    It 'fails when tag already exists'
      When call create_tag 2
      The error should include "Tag 'v2.0.0' already exists."
      The status should be failure
    End

    It 'creates tag with major.minor format'
      When call create_tag 2.1
      The output should include "Tag 'v2.1.0' created successfully."
      The contents of file VERSION should eq "2.1.0"
    End

    It 'creates tag with --no-v flag'
      When call create_tag 3.0.0 --no-v
      The output should include "Tag '3.0.0' created successfully."
      The contents of file VERSION should eq "3.0.0"
    End

    It 'creates predictive tag when no version is specified'
      echo "new feature" >> README.md
      git add README.md
      git commit -m "feat(api): add new endpoint"

      When call create_tag
      The output should include "Tag 'v3.1.0' created successfully."
      The contents of file VERSION should eq "3.1.0"
    End

    It 'creates tag with custom message using -m'
      When call create_tag 4.0.0 -m "Custom major release"
      The output should include "Tag 'v4.0.0' created successfully."
      The contents of file VERSION should eq "4.0.0"
    End

    It 'rejects invalid version format'
      When call create_tag "invalid_version"
      The error should include "Error: Invalid version format"
      The status should be failure
    End
  End
End

# Include the test helper
. "$SHELLSPEC_PROJECT_ROOT/spec/spec_helper.sh"

Describe 'version'
  Include "$SHELLSPEC_PROJECT_ROOT/src/utils/version.sh"

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

  Describe 'generate_version_info'
    BeforeEach 'rm -f VERSION'

    It 'creates VERSION file'
      When call generate_version_info
      The output should eq "Version information written to VERSION file"
      The file VERSION should be exist
      cat VERSION
    End

    It 'includes version tag'
      When call generate_version_info
      The output should be present
      The contents of file VERSION should include "v1.0.0"
    End

    It 'includes commit count'
      # Add a new commit
      echo "test" >> test.txt
      git add test.txt
      git commit -m "test commit"
      
      When call generate_version_info
      The output should be present
      The contents of file VERSION should match pattern "v1.0.0+1.*"
    End

    It 'includes date'
      current_date="$(date +%Y%m%d)"
      When call generate_version_info
      The output should be present
      The contents of file VERSION should include "$current_date"
    End

    It 'includes commit hash'
      commit_hash="$(git log -n 2 --format=%h | tail -n 1)"
      When call generate_version_info
      The output should be present
      The contents of file VERSION should include "$commit_hash"
    End
  End
End 
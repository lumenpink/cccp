# Include the test helper
. "$SHELLSPEC_PROJECT_ROOT/spec/spec_helper.sh"

Describe 'version'
  Include "${CCCP_BUNDLE:-$SHELLSPEC_PROJECT_ROOT/src/utils/version.sh}"

  setup() {
    TEST_DIR="$(mktemp -d)"
    cd "$TEST_DIR"
    git init
    git config user.email "test@example.com"
    git config user.name "Test User"
    touch README.md
    git add README.md
    git commit -m "initial commit"
    git tag -a "v1.0.0" -m "Version 1.0.0"
  }

  cleanup() {
    rm -rf "$TEST_DIR"
  }

  BeforeAll 'setup'
  AfterAll 'cleanup'

  Describe 'calculate_target_version'
    setup_calc() {
      CALC_DIR="$(mktemp -d)"
      cd "$CALC_DIR"
      git init
      git config user.email "test@example.com"
      git config user.name "Test User"
      touch README.md
      git add README.md
      git commit -m "initial commit"
    }

    cleanup_calc() {
      rm -rf "$CALC_DIR"
    }

    BeforeAll 'setup_calc'
    AfterAll 'cleanup_calc'

    It 'calculates next version from DEFAULT_BASE_VERSION 0.0.1 on initial commits'
      When call calculate_target_version
      The output should eq "0.0.2"
    End

    It 'bumps MINOR on untagged repository with feat commit'
      echo "untagged feat" >> file.txt
      git add file.txt
      git commit -m "feat: initial feature without tags"
      When call calculate_target_version
      The output should eq "0.1.0"
    End

    It 'bumps MAJOR on untagged repository with breaking change commit'
      echo "untagged breaking" >> file.txt
      git add file.txt
      git commit -m "feat!: initial breaking change without tags"
      When call calculate_target_version
      The output should eq "1.0.0"
    End

    It 'fails when calculate_target_version is run outside a git repository'
      NON_GIT_DIR="$(mktemp -d)"
      cd "$NON_GIT_DIR"
      When call calculate_target_version
      The status should be failure
      The error should include "Error: Not a git repository"
      cd "$CALC_DIR"
      rm -rf "$NON_GIT_DIR"
    End

    It 'bumps PATCH on fix commit'
      git tag -a "v1.0.0" -m "v1.0.0"
      echo "fix" >> file.txt
      git add file.txt
      git commit -m "fix: resolve small issue"
      When call calculate_target_version
      The output should eq "1.0.1"
    End

    It 'bumps MINOR on feat commit'
      echo "feat" >> file.txt
      git add file.txt
      git commit -m "feat: add new feature"
      When call calculate_target_version
      The output should eq "1.1.0"
    End

    It 'bumps MAJOR on breaking change marker !'
      echo "breaking" >> file.txt
      git add file.txt
      git commit -m "feat!: breaking API change"
      When call calculate_target_version
      The output should eq "2.0.0"
    End

    It 'bumps MAJOR on BREAKING CHANGE in body'
      echo "breaking_body" >> file.txt
      git add file.txt
      git commit -m "chore: some change" -m "BREAKING CHANGE: changes everything"
      When call calculate_target_version
      The output should eq "2.0.0"
    End

    It 'targets base version when latest tag is a pre-release'
      git tag -a "v2.0.0-dev" -m "v2.0.0-dev"
      echo "feat in dev" >> file.txt
      git add file.txt
      git commit -m "feat: new feature for v2"
      When call calculate_target_version
      The output should eq "2.0.0"
    End

    It 'maintains major version on breaking change in X.0.0 pre-release'
      echo "breaking in 2.0.0-dev" >> file.txt
      git add file.txt
      git commit -m "feat!: breaking architectural rewrite"
      When call calculate_target_version
      The output should eq "2.0.0"
    End

    It 'promotes to new major when breaking change occurs in non-X.0.0 pre-release'
      git tag -a "v2.1.0-dev" -m "v2.1.0-dev"
      echo "breaking in 2.1.0-dev" >> file.txt
      git add file.txt
      git commit -m "feat!: breaking change during minor dev"
      When call calculate_target_version
      The output should eq "3.0.0"
    End

    It 'promotes patch pre-release to minor when feature commit is added'
      git tag -a "v2.0.1-dev" -m "v2.0.1-dev"
      echo "feat in patch dev" >> file.txt
      git add file.txt
      git commit -m "feat: unplanned feature in patch dev"
      When call calculate_target_version
      The output should eq "2.1.0"
    End
  End

  Describe 'generate_version_info'
    BeforeEach 'rm -f VERSION'

    It 'creates VERSION file'
      When call generate_version_info
      The output should include "Version information written to VERSION file"
      The file VERSION should be exist
      cat VERSION
    End

    It 'produces clean SemVer when exactly on a tag (0 commits ahead)'
      When call generate_version_info
      The output should include "Version information written to VERSION file: 1.0.0"
      The contents of file VERSION should eq "1.0.0"
    End

    It 'includes predictive dev format when commits ahead'
      echo "test" >> test.txt
      git add test.txt
      git commit -m "feat: add shiny feature"
      
      When call generate_version_info
      The output should be present
      The contents of file VERSION should match pattern "1.1.0-dev.1+*"
    End

    It 'includes date in development version string'
      current_date="$(date +%Y%m%d)"
      When call generate_version_info
      The output should be present
      The contents of file VERSION should include "$current_date"
    End

    It 'includes commit hash in development version string'
      commit_hash="$(git rev-parse --short HEAD)"
      When call generate_version_info
      The output should be present
      The contents of file VERSION should include "$commit_hash"
    End

    It 'fails when generate_version_info is run outside a git repository'
      NON_GIT_DIR="$(mktemp -d)"
      cd "$NON_GIT_DIR"
      When call generate_version_info
      The status should be failure
      The error should include "Error: Not a git repository"
      cd "$TEST_DIR"
      rm -rf "$NON_GIT_DIR"
    End
  End
End
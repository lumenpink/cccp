# Include the test helper
. "$SHELLSPEC_PROJECT_ROOT/spec/spec_helper.sh"

Describe 'interactive'
  Include "${CCCP_BUNDLE:-$SHELLSPEC_PROJECT_ROOT/src/utils/interactive.sh}"
  Include "${CCCP_BUNDLE:-$SHELLSPEC_PROJECT_ROOT/src/utils/commit.sh}"
  Include "${CCCP_BUNDLE:-$SHELLSPEC_PROJECT_ROOT/src/utils/validation.sh}"
  Include "${CCCP_BUNDLE:-$SHELLSPEC_PROJECT_ROOT/src/utils/config_manager.sh}"

  setup() {
    TEST_DIR="$(mktemp -d)"
    cd "$TEST_DIR"

    git init
    git config user.email "test@example.com"
    git config user.name "Test User"
    touch README.md
    git add README.md
    git commit -m "chore: initial commit"
  }

  cleanup() {
    rm -rf "$TEST_DIR"
  }

  BeforeEach 'setup'
  AfterEach 'cleanup'

  Describe 'interactive_commit'
    It 'fails when no changes are staged for commit'
      When call interactive_commit
      The status should be failure
      The error should include "No changes staged for commit"
    End

    It 'fails when executed outside of a git repository'
      NON_GIT_DIR="$(mktemp -d)"
      cd "$NON_GIT_DIR"
      When call interactive_commit
      The status should be failure
      The error should include "Error: Not a git repository"
      rm -rf "$NON_GIT_DIR"
    End

    It 'successfully creates a standard conventional commit from wizard prompts'
      echo "feature code" >> README.md
      git add README.md

      # Inputs: choice 1 (feat), scope "cli", subject "add wizard", breaking "n", body "", confirm "y"
      run_wizard() {
        printf "1\ncli\nadd wizard\nn\n\ny\n" | interactive_commit
        echo "MSG:$(git log -1 --format='%s')"
      }

      When call run_wizard
      The status should be success
      The output should include "★ CCCP Assembly Line (Interactive Soviet Wizard) ★"
      The output should include "Commit recorded into the State archives successfully"
      The output should include "MSG:feat(cli): add wizard"
    End

    It 'supports breaking change marker and extended body'
      echo "breaking change" >> README.md
      git add README.md

      # Inputs: choice 2 (fix), scope "api", subject "drop v1", breaking "y", body "removed legacy routes", confirm "y"
      run_wizard_breaking() {
        printf "2\napi\ndrop v1\ny\nremoved legacy routes\ny\n" | interactive_commit
        echo "MSG:$(git log -1 --format='%s')"
        echo "BODY:$(git log -1 --format='%b')"
      }

      When call run_wizard_breaking
      The status should be success
      The output should include "MSG:fix(api)!: drop v1"
      The output should include "BODY:removed legacy routes"
    End

    It 'aborts commit when user enters n at confirmation'
      echo "aborted change" >> README.md
      git add README.md

      run_wizard_abort() {
        printf "1\ncore\nabort test\nn\n\nn\n" | interactive_commit
      }

      When call run_wizard_abort
      The status should be failure
      The output should include "Commit aborted by worker directive"
    End

    It 'is invoked by commit -i'
      echo "wizard commit" >> README.md
      git add README.md

      run_commit_flag() {
        printf "1\nui\nsupport wizard\nn\n\ny\n" | commit -i
        echo "MSG:$(git log -1 --format='%s')"
      }

      When call run_commit_flag
      The status should be success
      The output should include "MSG:feat(ui): support wizard"
    End

    It 'creates commit without scope when user skips scope prompt with Enter'
      echo "no scope change" >> README.md
      git add README.md

      run_wizard_no_scope() {
        printf "1\n\nadd without scope\nn\n\ny\n" | interactive_commit
        echo "MSG:$(git log -1 --format='%s')"
      }

      When call run_wizard_no_scope
      The status should be success
      The output should include "MSG:feat: add without scope"
    End

    It 'selects type by entering type name directly'
      echo "type by name change" >> README.md
      git add README.md

      run_wizard_type_name() {
        printf "fix\ncli\npatch parser\nn\n\ny\n" | interactive_commit
        echo "MSG:$(git log -1 --format='%s')"
      }

      When call run_wizard_type_name
      The status should be success
      The output should include "MSG:fix(cli): patch parser"
    End

    It 'supports custom types and scopes loaded from .cccprc'
      cat > "$TEST_DIR/.cccprc" << 'EOF'
commit_types = "feat,fix,party"
commit_scopes = "kremlin,sputnik"
EOF
      echo "custom soviet change" >> README.md
      git add README.md

      run_wizard_custom_config() {
        printf "3\nkremlin\nstate directive\nn\n\ny\n" | interactive_commit
        echo "MSG:$(git log -1 --format='%s')"
      }

      When call run_wizard_custom_config
      The status should be success
      The output should include "MSG:party(kremlin): state directive"
      rm -f "$TEST_DIR/.cccprc"
    End

    It 'recovers from invalid type selection and prompts again until valid'
      echo "retry type change" >> README.md
      git add README.md

      run_wizard_retry_type() {
        printf "99\ninvalid_type\n1\n\nrecover from invalid type\nn\n\ny\n" | interactive_commit
        echo "MSG:$(git log -1 --format='%s')"
      }

      When call run_wizard_retry_type
      The status should be success
      The output should include "Invalid selection. Please choose a number"
      The output should include "MSG:feat: recover from invalid type"
    End

    It 'recovers from empty subject and prompts again until provided'
      echo "retry subject change" >> README.md
      git add README.md

      run_wizard_retry_subject() {
        printf "1\n\n   \n\nvalid subject after empty\nn\n\ny\n" | interactive_commit
        echo "MSG:$(git log -1 --format='%s')"
      }

      When call run_wizard_retry_subject
      The status should be success
      The output should include "Commit subject cannot be empty."
      The output should include "MSG:feat: valid subject after empty"
    End

    It 'aborts cleanly when input stream terminates during type selection'
      echo "eof change" >> README.md
      git add README.md

      run_wizard_eof_type() {
        printf "" | interactive_commit
      }

      When call run_wizard_eof_type
      The status should be failure
      The output should include "★ CCCP Assembly Line (Interactive Soviet Wizard) ★"
    End

    It 'aborts cleanly when input stream terminates during subject entry'
      echo "eof subject change" >> README.md
      git add README.md

      run_wizard_eof_subject() {
        printf "1\ncore\n" | interactive_commit
      }

      When call run_wizard_eof_subject
      The status should be failure
      The output should include "Step 3: Commit subject"
    End

    It 'fails when validation rejects message under strict scopes constraint'
      echo "strict scope change" >> README.md
      git add README.md

      run_wizard_strict_failure() {
        STRICT_SCOPES=1 COMMIT_SCOPES="core,cli"
        export STRICT_SCOPES COMMIT_SCOPES
        printf "1\nunauthorized_scope\ntest strict\nn\n\ny\n" | interactive_commit
      }

      When call run_wizard_strict_failure
      The status should be failure
      The output should include "Proposed State Commit"
      The error should include "Validation failed on constructed message"
    End

    It 'accepts confirmation with full word yes'
      echo "yes confirmation change" >> README.md
      git add README.md

      run_wizard_yes_word() {
        printf "1\n\nconfirm with yes word\nn\n\nyes\n" | interactive_commit
        echo "MSG:$(git log -1 --format='%s')"
      }

      When call run_wizard_yes_word
      The status should be success
      The output should include "MSG:feat: confirm with yes word"
    End
  End
End


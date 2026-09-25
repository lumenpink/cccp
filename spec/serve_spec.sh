# Include the test helper
. "$SHELLSPEC_PROJECT_ROOT/spec/spec_helper.sh"

Describe 'Unified Server Engine'
  Include "${CCCP_BUNDLE:-$SHELLSPEC_PROJECT_ROOT/src/utils/serve.sh}"

  setup() {
    TEST_WORKSPACE="$(mktemp -d)"
    export NO_COLOR="1"
  }

  cleanup() {
    rm -rf "$TEST_WORKSPACE"
    unset NO_COLOR
  }

  BeforeEach 'setup'
  AfterEach 'cleanup'

  Describe '_serve_detect_target_dir'
    It 'returns explicit directory when passed and exists'
      mkdir -p "$TEST_WORKSPACE/custom_web"
      When call _serve_detect_target_dir "$TEST_WORKSPACE/custom_web"
      The status should be success
      The output should eq "$TEST_WORKSPACE/custom_web"
    End

    It 'detects ./site when present'
      _detect_site() {
        cd "$TEST_WORKSPACE"
        mkdir -p "site"
        _serve_detect_target_dir ""
      }
      When call _detect_site
      The status should be success
      The output should eq "./site"
    End

    It 'detects ./dist when site is absent'
      _detect_dist() {
        cd "$TEST_WORKSPACE"
        mkdir -p "dist"
        _serve_detect_target_dir ""
      }
      When call _detect_dist
      The status should be success
      The output should eq "./dist"
    End

    It 'detects ./public when site and dist are absent'
      _detect_public() {
        cd "$TEST_WORKSPACE"
        mkdir -p "public"
        _serve_detect_target_dir ""
      }
      When call _detect_public
      The status should be success
      The output should eq "./public"
    End

    It 'falls back to . when no standard web folder exists'
      _detect_dot() {
        cd "$TEST_WORKSPACE"
        _serve_detect_target_dir ""
      }
      When call _detect_dot
      The status should be success
      The output should eq "."
    End
  End

  Describe '_serve_find_free_port'
    It 'returns initial port when free'
      When call _serve_find_free_port 39182 "127.0.0.1"
      The status should be success
      The output should eq "39182"
    End
  End

  Describe '_serve_display_banner'
    It 'displays standard Gosplan banner for static mode'
      When call _serve_display_banner "static" "./site" "0.0.0.0" "8080"
      The status should be success
      The output should include "CCCP HTTP SERVER ENGINE"
      The output should include "http://localhost:8080/"
    End

    It 'displays VDNKh banner for demo mode'
      When call _serve_display_banner "demo" "./site" "0.0.0.0" "8080"
      The status should be success
      The output should include "ВДНХ"
      The output should include "EXPOSIÇÃO DAS CONQUISTAS DA ECONOMIA NACIONAL"
      The output should include "http://localhost:8080/"
    End
  End

  Describe 'cmd_serve options & delegation'
    It 'displays help message on -h or --help'
      When call cmd_serve --help
      The status should be success
      The output should include "Uso: cccp serve"
      The output should include "--dev"
      The output should include "--demo"
      The output should include "--localhost"
    End

    It 'delegates to scripts/dev.sh when present in dev mode'
      mkdir -p "$TEST_WORKSPACE/scripts"
      echo '#!/bin/sh' > "$TEST_WORKSPACE/scripts/dev.sh"
      echo 'echo "DELEGATED_DEV_RUNNER"' >> "$TEST_WORKSPACE/scripts/dev.sh"
      chmod +x "$TEST_WORKSPACE/scripts/dev.sh"

      _run_dev_test() {
        cd "$TEST_WORKSPACE"
        cmd_serve --dev
      }
      When call _run_dev_test
      The status should be success
      The output should include "Delegando execução para scripts/dev.sh"
      The output should include "DELEGATED_DEV_RUNNER"
    End
  End
End

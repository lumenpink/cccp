#!/bin/sh

. "$SHELLSPEC_PROJECT_ROOT/spec/spec_helper.sh"

Describe 'comrades'
  Include "${CCCP_BUNDLE:-$SHELLSPEC_PROJECT_ROOT/src/utils/comrades.sh}"

  setup() {
    TEST_DIR="$(mktemp -d)"
    cd "$TEST_DIR"
  }

  cleanup() {
    rm -rf "$TEST_DIR"
  }

  BeforeEach 'setup'
  AfterEach 'cleanup'

  Describe '_cccp_detect_arch'
    It 'detects current system architecture string'
      When call _cccp_detect_arch
      The status should be success
      The output should match pattern "*-*"
    End

    It 'normalizes darwin uname to macos'
      uname() {
        case "$1" in
          -s) echo "Darwin" ;;
          -m) echo "arm64" ;;
          *) echo "unknown" ;;
        esac
      }
      When call _cccp_detect_arch
      The status should be success
      The output should eq "macos-aarch64"
    End

    It 'normalizes linux x86_64'
      uname() {
        case "$1" in
          -s) echo "Linux" ;;
          -m) echo "x86_64" ;;
          *) echo "unknown" ;;
        esac
      }
      When call _cccp_detect_arch
      The status should be success
      The output should eq "linux-x86_64"
    End

    It 'normalizes 32-bit i686 architecture'
      uname() {
        case "$1" in
          -s) echo "Linux" ;;
          -m) echo "i686" ;;
          *) echo "unknown" ;;
        esac
      }
      When call _cccp_detect_arch
      The status should be success
      The output should eq "linux-i686"
    End

    It 'normalizes 32-bit armv7 architecture'
      uname() {
        case "$1" in
          -s) echo "Linux" ;;
          -m) echo "armv7l" ;;
          *) echo "unknown" ;;
        esac
      }
      When call _cccp_detect_arch
      The status should be success
      The output should eq "linux-armv7"
    End

    It 'normalizes 32-bit armv6l, armhf and armv5tejl to armv7'
      uname() {
        case "$1" in
          -s) echo "Linux" ;;
          -m) echo "armv6l" ;;
          *) echo "unknown" ;;
        esac
      }
      When call _cccp_detect_arch
      The status should be success
      The output should eq "linux-armv7"
    End

    It 'normalizes generic arm architecture with CPU hint fallback'
      uname() {
        case "$1" in
          -s) echo "Linux" ;;
          -m) echo "armv5tejl" ;;
          *) echo "unknown" ;;
        esac
      }
      When call _cccp_detect_arch
      The status should be success
      The output should eq "linux-armv7"
    End
  End

  Describe '_cccp_get_bin_dir'
    It 'resolves and creates a valid binary directory in PATH'
      export HOME="$TEST_DIR"
      export XDG_BIN_HOME="$TEST_DIR/bin"
      When call _cccp_get_bin_dir
      The status should be success
      The output should eq "$TEST_DIR/bin"
      The path "$TEST_DIR/bin" should be directory
      The variable PATH should include "$TEST_DIR/bin"
    End

    It 'falls back to XDG_DATA_HOME/cccp/bin when local/bin cannot be used'
      export HOME="$TEST_DIR"
      unset XDG_BIN_HOME || true
      export XDG_DATA_HOME="$TEST_DIR/data"
      # Mock ~/.local/bin to be a file so mkdir fails
      mkdir -p "$TEST_DIR/.local"
      touch "$TEST_DIR/.local/bin"
      When call _cccp_get_bin_dir
      The status should be success
      The output should eq "$TEST_DIR/data/cccp/bin"
      The path "$TEST_DIR/.cccp" should not be exist
      rm -f "$TEST_DIR/.local/bin"
    End

    It 'falls back to HOME/.local/share/cccp/bin when XDG_DATA_HOME is unset and local/bin cannot be used'
      export HOME="$TEST_DIR"
      unset XDG_BIN_HOME XDG_DATA_HOME || true
      mkdir -p "$TEST_DIR/.local"
      touch "$TEST_DIR/.local/bin"
      When call _cccp_get_bin_dir
      The status should be success
      The output should eq "$TEST_DIR/.local/share/cccp/bin"
      The path "$TEST_DIR/.cccp" should not be exist
      rm -f "$TEST_DIR/.local/bin"
    End
  End

  Describe '_cccp_ensure_comrade'
    It 'rejects unknown comrades'
      When call _cccp_ensure_comrade "unknown_tool" 0
      The status should be failure
      The error should include "Camarada desconhecido"
    End

    It 'downloads and installs comrade tarball'
      export HOME="$TEST_DIR"
      export XDG_BIN_HOME="$TEST_DIR/bin"
      export PATH="$TEST_DIR/bin:/bin:/usr/bin"
      mkdir -p "$TEST_DIR/bin"

      # Mock download function
      _cccp_download_raw() {
        _dest="$2"
        _mock_src="$(mktemp -d)"
        echo '#!/bin/sh' > "$_mock_src/kv"
        echo 'echo "kv v1.0.0"' >> "$_mock_src/kv"
        chmod +x "$_mock_src/kv"
        tar -czf "$_dest" -C "$_mock_src" kv
        rm -rf "$_mock_src"
        return 0
      }

      When call _cccp_ensure_comrade "kv" 0
      The status should be success
      The output should include "ГОСПЛАН"
      The output should include "recrutado e instalado"
      The path "$TEST_DIR/bin/kv" should be executable
    End

    It 'prioritizes CCCP_COMRADES_MIRROR when configured'
      export HOME="$TEST_DIR"
      export XDG_BIN_HOME="$TEST_DIR/bin"
      export PATH="$TEST_DIR/bin:/bin:/usr/bin"
      export CCCP_COMRADES_MIRROR="http://router.local/packages"

      _cccp_download_raw() {
        echo "DOWNLOAD_URL: $1" > "$TEST_DIR/attempted_url"
        _dest="$2"
        _mock_src="$(mktemp -d)"
        echo '#!/bin/sh' > "$_mock_src/kv"
        chmod +x "$_mock_src/kv"
        tar -czf "$_dest" -C "$_mock_src" kv
        rm -rf "$_mock_src"
        return 0
      }

      When call _cccp_ensure_comrade "kv" 1
      The status should be success
      The contents of file "$TEST_DIR/attempted_url" should include "http://router.local/packages/kv-"
    End

    It 'maps mk/mayak to mayak repo'
      export HOME="$TEST_DIR"
      export XDG_BIN_HOME="$TEST_DIR/bin"
      export PATH="$TEST_DIR/bin:/bin:/usr/bin"
      mkdir -p "$TEST_DIR/bin"

      _cccp_download_raw() {
        echo "URL: $1" > "$TEST_DIR/attempted_url"
        _dest="$2"
        _mock_src="$(mktemp -d)"
        echo '#!/bin/sh' > "$_mock_src/mk"
        chmod +x "$_mock_src/mk"
        tar -czf "$_dest" -C "$_mock_src" mk
        rm -rf "$_mock_src"
        return 0
      }

      When call _cccp_ensure_comrade "mk" 1
      The status should be success
      The contents of file "$TEST_DIR/attempted_url" should include "/mayak/releases/download/nightly/mk-"
      The path "$TEST_DIR/bin/mk" should be executable
    End
  End

  Describe '_cccp_download_raw'
    It 'uses openssl s_client when sv, curl, wget and busybox are absent'
      openssl() {
        if [ "$1" = "s_client" ]; then
          printf 'HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\n\r\nmocked binary payload\n'
          return 0
        fi
        return 1
      }
      curl() { return 127; }
      wget() { return 127; }
      sv() { return 127; }
      svyaz() { return 127; }
      busybox() { return 127; }

      out_file="$TEST_DIR/downloaded.bin"
      When call _cccp_download_raw "https://codeberg.org/lumenpink/svyaz/releases/download/nightly/sv-linux-i686.tar.gz" "$out_file"
      The status should be success
      The contents of file "$out_file" should eq "mocked binary payload"
    End

    It 'uses netcat (nc) for HTTP download on OpenWrt routers without SSL'
      nc() {
        printf 'HTTP/1.0 200 OK\r\nContent-Type: application/octet-stream\r\n\r\nopenwrt-nc-payload\n'
        return 0
      }
      curl() { return 127; }
      wget() { return 127; }
      sv() { return 127; }
      svyaz() { return 127; }
      busybox() { return 127; }

      out_file="$TEST_DIR/nc_downloaded.bin"
      When call _cccp_download_raw "http://192.168.1.1/packages/sv-linux-armv7.tar.gz" "$out_file"
      The status should be success
      The contents of file "$out_file" should eq "openwrt-nc-payload"
    End

    It 'uses telnet (1960 style) for HTTP download when nc is absent'
      nc() { return 127; }
      telnet() {
        printf 'HTTP/1.0 200 OK\r\nContent-Type: application/octet-stream\r\n\r\ntelnet-1960-payload\n'
        return 0
      }
      curl() { return 127; }
      wget() { return 127; }
      sv() { return 127; }
      svyaz() { return 127; }
      busybox() { return 127; }

      out_file="$TEST_DIR/telnet_downloaded.bin"
      When call _cccp_download_raw "http://192.168.1.1/packages/kv-linux-armv7.tar.gz" "$out_file"
      The status should be success
      The contents of file "$out_file" should eq "telnet-1960-payload"
    End

    It 'uses ftpget for FTP URLs'
      ftpget() {
        printf 'ftpget-payload\n' > "$2"
        return 0
      }
      curl() { return 127; }
      wget() { return 127; }
      sv() { return 127; }
      svyaz() { return 127; }
      busybox() { return 127; }

      out_file="$TEST_DIR/ftp_downloaded.bin"
      When call _cccp_download_raw "ftp://mirror.soviet.su/pub/sv-linux-armv7.tar.gz" "$out_file"
      The status should be success
      The contents of file "$out_file" should eq "ftpget-payload"
    End
  End

  Describe '_cccp_http'
    It 'respects CCCP_HTTP_CLIENT override'
      mock_http() {
        echo "CUSTOM_HTTP: $*"
      }
      export CCCP_HTTP_CLIENT="mock_http"
      When call _cccp_http -s "https://example.com"
      The status should be success
      The output should eq "CUSTOM_HTTP: -s https://example.com"
    End

    It 'dispatches to mocked curl function'
      curl() {
        echo "MOCKED_CURL: $*"
      }
      When call _cccp_http -s "https://api.github.com"
      The status should be success
      The output should eq "MOCKED_CURL: -s https://api.github.com"
    End

    It 'dispatches to mocked sv function'
      sv() {
        echo "MOCKED_SV: $*"
      }
      When call _cccp_http -s "https://codeberg.org"
      The status should be success
      The output should eq "MOCKED_SV: -s https://codeberg.org"
    End
  End

  Describe 'cmd_comrades'
    It 'displays help information on --help'
      When call cmd_comrades "--help"
      The status should be success
      The output should include "CCCP - Gestão de Camaradas (Kvant e Svyaz)"
      The output should include "cccp comrades [status]"
      The output should include "cccp comrades install"
      The output should include "cccp comrades update"
    End

    It 'displays comrade status'
      export HOME="$TEST_DIR"
      export XDG_BIN_HOME="$TEST_DIR/bin"
      mkdir -p "$TEST_DIR/bin"
      cat > "$TEST_DIR/bin/kv" << 'EOF'
#!/bin/sh
echo "kv v0.1.0"
EOF
      chmod +x "$TEST_DIR/bin/kv"
      export PATH="$TEST_DIR/bin:$PATH"

      When call cmd_comrades "status"
      The status should be success
      The output should include "ГОСПЛАН"
      The output should include "kv"
      The output should include "Editor (kvant)"
      The output should include "sv"
      The output should include "HTTP (svyaz)"
    End

    It 'installs comrades via install command'
      _cccp_ensure_comrade() {
        echo "CALLED_ENSURE: $1 (silent: $2)"
      }
      When call cmd_comrades "install" "kv"
      The status should be success
      The output should include "CALLED_ENSURE: kv (silent: 0)"
    End

    It 'fails on invalid subcommand'
      When call cmd_comrades "invalid_action"
      The status should be failure
      The error should include "Uso: cccp comrades"
    End
  End

  Describe 'zero-awk compliance'
    It 'verifies zero executable awk invocations in src/utils/comrades.sh'
      # Remove comments and strings, then check for awk command invocations
      grep_awk() {
        grep -E '^[[:space:]]*awk[[:space:]]|;[[:space:]]*awk[[:space:]]|\|[[:space:]]*awk[[:space:]]|\$\(awk[[:space:]]' "$SHELLSPEC_PROJECT_ROOT/src/utils/comrades.sh" || true
      }
      When call grep_awk
      The output should eq ""
    End
  End
End

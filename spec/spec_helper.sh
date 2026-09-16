# Set the shell options for testing
set -e

# Dual-target testing configuration: modular (default) or bundled
CCCP_TEST_TARGET="${CCCP_TEST_TARGET:-modular}"
export CCCP_TEST_TARGET

if [ "$CCCP_TEST_TARGET" = "bundled" ]; then
  export CCCP_SOURCED=1
  export CCCP_BUNDLE="$SHELLSPEC_PROJECT_ROOT/cccp.sh"
  export CCCP_BIN="$SHELLSPEC_PROJECT_ROOT/cccp.sh"
else
  export CCCP_BUNDLE=""
  export CCCP_BIN="$SHELLSPEC_PROJECT_ROOT/bin/cccp"
  # Include the configuration for modular mode
  . "$SHELLSPEC_PROJECT_ROOT/src/config/config.sh"
fi

cd "$SHELLSPEC_PROJECT_ROOT"

export GIT_HOOK_FILE="cccp-base.sh"
export CHECK_UPDATES=0

# Helper function to create a test git repository
setup_test_repository() {
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
  
  echo "$TEST_DIR"
}

# Helper function to clean up test repository
cleanup_test_repository() {
  rm -rf "$1"
}

# Helper function to create a test commit
create_test_commit() {
  echo "test" >> test.txt
  git add test.txt
  git commit -m "$1"
} 
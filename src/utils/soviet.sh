#!/bin/sh

# Enable error handling
set -eu

# -----------------------------------------------------------------------------
# Soviet Easter Egg
# -----------------------------------------------------------------------------
show_soviet() {
    cat << 'EOF'
                 ★
              /-----\
             |  CCCP |   CONVENTIONAL COMMITS COMPLIANCE PROGRAM
              \-----/    "Workers of the World, Conventionalize! ☭"
             /   |   \
            /    |    \

★ COMRADE! The State Planning Committee (Gosplan) salutes your discipline!
★ ORDER NO. 227: Not one step back from Conventional Commits!
★ Five-Year Plan for 100% compliant Git history is fulfilling its quota!

Directives:
  • feat:     Industrial progress for the Motherland (Minor bump)
  • fix:      Repair sabotage in the machinery (Patch bump)
  • BREAKING: Revolutionary paradigm shift (Major bump)

Glory to the Standardized Commit History!
EOF
}

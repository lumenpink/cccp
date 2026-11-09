# Universal Phase 9 Resolution Report Headers (#151)

## Context & Motivation
In accordance with GOSPLAN-003 and internationalization requirements, resolution reports for Phase 9 must adopt universal canonical headers in English:
- `[Summary]:` (with backward-compatible support for `[Resumo]:`)
- `[Solution]:` (with backward-compatible support for `[Solução]:`)
- `[Validation]:` (with backward-compatible support for `[Validação]:`)

## Technical Specification
1. In `src/ddtdd.rs`:
   - Update `validate_resolution_report` to verify the presence of Summary, Solution, and Validation in either English or Portuguese.
   - Update the interactive template generated for Phase 9 to use canonical English headers.
2. In `src/cli.rs`:
   - Update CLI help text for `cccp close --body` to mention `[Summary]:`, `[Solution]:`, `[Validation]:`.

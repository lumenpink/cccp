# Project Directives: CCCP

> Conventional Commits Compliance Program - Semantic Versioning, Git Hooks & Changelog Manager

## 1. System & Operational Baseline
- **Primary Stack:** POSIX Shell, Git
- **Environment:** Treat active environment as standard Linux/POSIX. Read secrets strictly from local `.env` or environment variables; never hardcode credentials.
- **Verification Commands:**
  - **Tests:** `shellspec` (run before and after changes to guarantee zero regressions)
  - **Lint & Format:** `./combine.sh && sh -n cccp.sh`

## 2. Architecture & Conventions
- **Separation of Concerns:** Keep core domain logic strictly separated from transport layers, CLI adapters, and filesystem I/O.
- **Typing & Robustness:** Enforce strict typing. Handle all edge cases explicitly with actionable errors; never swallow exceptions or fail silently.
- **Clean Commits:** Follow Conventional Commits in English (`feat:`, `fix:`, `refactor:`, `chore:`). Keep changes minimal, atomic, and focused.

## 3. Strict Constraints
- Do not install external dependencies or modify runtime packages without explicit approval.
- Do not alter existing schema migrations or commit breaking contract changes.
- Do not make unsolicited architectural changes or cosmetic rewrites outside the designated task scope.
## 4. Specific Directives
- Follow strict Gosplan and Soviet aesthetic in cultural commands and easter eggs.
- Never commit automatically unless explicitly instructed by user.

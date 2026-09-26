# Active Tickets - cccp
> Synced at: 2026-09-26 10:59:19 UTC
> Ceiling: 10240 bytes (strictly uncut tickets)

## [#37] Implement pre-push hook with act: local CI gate, live spinner and structured logs
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/37

### Description
## Problem
Pushes can reach the remote forge with failing tests. Running `composer test` or `cargo test` locally is insufficient — local environments diverge from CI (runtime versions, env vars, system deps), producing the classic 'passes locally, fails on CI' problem.

## Expected Behavior
The `pre-push` hook installed by `cccp install` should:

### Core behavior
1. Detect if `act` (Docker-based Forgejo/GitHub Actions runner) is available
2. Run `act -j test --no-pull` against the repo's `.forgejo/workflows/ci.yml` (using cached images)
3. Stream full output to a structured log file: `.git/act-logs/<YYYY-MM-DD>_<HH-MM>_<branch>_<short-sha>.log`
4. Show live feedback in the terminal (spinner, current step name, elapsed time)
5. On success: print summary and allow the push to proceed
6. On failure: print condensed error summary with log path, abort the push

### UX (terminal output during gate)
```
⠸ [pre-push] Running CI gate locally...
  Job: test › Lint & Static Analysis     [0:12]
  Log: .git/act-logs/2026-09-24_09-41_feat-42_a4b3c2d.log
```

On pass:
```
✅ CI gate passed (1m 34s) — pushing to origin/main
```

On fail:
```
❌ CI gate failed (0m 47s) — push aborted
   → Step failed: cargo test
   → Log: .git/act-logs/2026-09-24_09-41_feat-42_a4b3c2d.log
   → Bypass (conscious): git push --no-verify
```

### Command: cccp act log
Shortcut to open/tail the most recent act log file.

### Opt-out
`git push --no-verify` is the documented conscious bypass.

## Acceptance Criteria
- [ ] pre-push hook runs `act -j test` before allowing push
- [ ] act stdout/stderr fully captured in `.git/act-logs/`
- [ ] Log filename includes timestamp, branch name and short sha
- [ ] Live spinner showing current step and elapsed time
- [ ] Clear pass/fail summary with log path
- [ ] `cccp act log` opens/tails the latest log
- [ ] Graceful degradation if `act` or Docker is not running (warn, don't block)
- [ ] Documented in `cccp help`

---

# Predictive Semantic Versioning

> **«ПЯТИЛЕТКУ — В ЧЕТЫРЕ ГОДА!»**  
> *(The Five-Year Plan in four years! — Accelerate production!)*

CCCP delivers automated, deterministic Semantic Versioning (SemVer 2.0.0) without requiring heavyweight language runtimes or complex CI configurations.

---

## 1. Version Format

CCCP maintains two distinct version formats:

### Clean SemVer (Release)
When the working commit exactly matches a Git release tag:
```text
MAJOR.MINOR.PATCH
Example: 2.1.0
```

### Predictive Pre-Release (Development)
When developing between release tags, CCCP dynamically calculates the prospective target version and attaches build metadata:
```text
<TARGET_SEMVER>-dev.<COMMITS_AHEAD>+<DATE>.<SHORT_HASH>
Example: 2.2.0-dev.4+20260921.a97058e
```

- `<TARGET_SEMVER>`: Predicted release version calculated by scanning all commits since the last release tag.
- `dev.<COMMITS_AHEAD>`: The exact number of unreleased commits since the last tag.
- `<DATE>`: Build date (`YYYYMMDD`).
- `<SHORT_HASH>`: Abbreviated Git commit SHA.

---

## 2. Version Bump Triggers

CCCP evaluates every commit message since the previous tag:

| Commit Signal | SemVer Component Bumped | Meaning & Soviet Directive |
| :--- | :--- | :--- |
| `!` after type/scope, or `BREAKING CHANGE:` in body/footer | **MAJOR** (`X.0.0`) | Revolutionary paradigm shift |
| `feat:` / `feat(scope):` | **MINOR** (`x.Y.0`) | Industrial progress for the Motherland |
| `fix:`, `perf:`, `refactor:`, `chore:`, etc. | **PATCH** (`x.y.Z`) | Machinery maintenance & repairs |

---

## 3. Creating Release Tags (`cccp tag`)

When your development cycle is complete, promote your project to a clean release:

```bash
# Calculate next version automatically from commits and create tag:
cccp tag

# Explicitly bump to a specific version:
cccp tag 2.3.0

# Add a custom release title and message:
cccp tag --title "The Stakhanovite Release" -m "Delivered ahead of quota"
```

The `cccp tag` command automatically:
1. Validates that working tree is clean.
2. Updates `VERSION` to the clean release version.
3. Finalizes `CHANGELOG.md` with release date and grouped commits.
4. Generates an annotated, signed (if configured) Git tag.

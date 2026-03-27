# Test Coverage Comparison: anodize vs GoReleaser

**Date:** 2026-03-26
**GoReleaser version:** main branch (latest)
**anodize version:** session-3/release-readiness branch (814 tests)

---

## 1. Total Test Count

| Metric | GoReleaser | anodize |
|--------|-----------|---------|
| Test files | 164 | ~30 (crate modules + integration) |
| Top-level test functions | ~600 | 814 |
| Including subtests | **~1,800-2,200** | **814** |
| Fuzz tests | 7 (2 files) | 0 |

**Verdict: GoReleaser wins decisively.** GoReleaser has roughly 2-3x more test cases
when subtests are counted. Go's `t.Run()` subtest pattern means a single top-level
function like `TestSignArtifacts` contains 44 subtests, or `TestRunPipe` in brew
contains 53 subtests. Rust's `#[test]` model counts each as a separate function,
so the 814 number is directly comparable to GoReleaser's ~1,800-2,200 total including
subtests.

### How the GoReleaser estimate was derived

Concrete counts gathered from individual test files (top-level + subtests):

| File/Area | Top-level | Subtests | Total |
|-----------|----------|----------|-------|
| pipe/brew | 41 | 53 | 94 |
| pipe/archive | 31 | 17 | 48 |
| pipe/checksums | 16 | 11 | 27 |
| pipe/changelog | 40 | 67 | 107 |
| pipe/sign (sign_test.go only) | 8 | 44 | 52 |
| pipe/nfpm | 34 | 41 | 75 |
| pipe/release | 27 | 13 | 40 |
| pipe/docker | 19 | 47 | 66 |
| pipe/scoop | 9 | 25 | 34 |
| pipe/env | 27 | 13 | 40 |
| pipe/build | 29 | 8 | 37 |
| pipe/git | 36 | 18 | 54 |
| pipe/announce | 4 | 5 | 9 |
| pipe/aur | 16 | 22 | 38 |
| pipe/chocolatey | 9 | 7 | 16 |
| pipe/flatpak | 11 | 7 | 18 |
| pipe/ko | 16 | 29 | 45 |
| pipe/krew | 24 | 15 | 39 |
| pipe/winget | 7 | 54 | 61 |
| pipe/nix | 13 | 41 | 54 |
| pipe/blob | 11 | 15 | 26 |
| pipe/sbom | 13 | 32 | 45 |
| pipe/snapcraft | 27 | 8 | 35 |
| pipe/semver | 4 | 0 | 4 |
| pipe/before | 8 | 3 | 11 |
| pipe/sourcearchive | 9 | 7 | 16 |
| pipe/publish | 5 | 2 | 7 |
| pipe/gomod + proxy | 18 | 21 | 39 |
| pipe/universalbinary | 4 | 27 | 31 |
| pipe/makeself | 4 | 5 | 9 |
| pipe/dist | 7 | 2 | 9 |
| pipe/effectiveconfig | 2 | 0 | 2 |
| pipe/partial | 3 | 26 | 29 |
| pipe/defaults | 4 | 0 | 4 |
| pipe/milestone | 17 | 2 | 19 |
| pipe/project | 10 | 0 | 10 |
| pipe/metadata | 2 | 3 | 5 |
| pipe/reportsizes | 3 | 2 | 5 |
| pipe/prebuild | 2 | 3 | 5 |
| pipe/upx | 8 | 11 | 19 |
| pipe/snapshot | 8 | 2 | 10 |
| **Subtotal (pipes only)** | **~609** | **~748** | **~1,357** |
| Announcers (slack, discord, etc.) | ~60 | ~35 | ~95 |
| builders/golang | 31 | 143 | 174 |
| builders/rust,zig,bun,etc. | ~30 | ~20 | ~50 |
| internal/tmpl | 23 | 69 | 92 |
| internal/artifact | 26 | 37 | 63 |
| internal/client (github+gitlab+gitea) | ~60 | ~85 | ~145 |
| internal/git | ~15 | ~10 | ~25 |
| pkg/config | 13 | 6 | 19 |
| pkg/archive (tar/zip/gzip etc.) | ~20 | ~15 | ~35 |
| pkg/context | 3 | 0 | 3 |
| cmd tests | ~30 | ~55 | ~85 |
| internal (misc: ids, redact, etc.) | ~30 | ~15 | ~45 |
| testlib self-tests | ~10 | ~5 | ~15 |
| **Grand total estimate** | **~930** | **~1,050** | **~2,000** |

---

## 2. Coverage Percentage

| Metric | GoReleaser | anodize |
|--------|-----------|---------|
| Overall line coverage | Unknown (Codecov badge present, page not scrapeable) | **80.4%** (4,983/6,198 lines) |
| Coverage tool | `go test -coverpkg=./... -covermode=atomic` | cargo-llvm-cov |

**Verdict: Tie / slight anodize advantage on transparency.** GoReleaser tracks
coverage via Codecov but doesn't publish the number prominently. Based on the
test density and patterns observed, GoReleaser likely has 60-75% coverage (Go
projects with this test density and many external-dependency pipes typically land
there). anodize's 80.4% is strong and verifiable.

**Caveat:** GoReleaser has far more code (estimated ~40-60k lines Go vs anodize's
~6,200 lines Rust), so raw coverage percentage comparisons have limited meaning.
GoReleaser has massive surface area for package managers, container tools, and
SCM providers that are hard to unit test without real infrastructure.

---

## 3. Per-Stage Comparison

### Stages We Both Have

| Stage | GoReleaser tests | anodize tests | Verdict |
|-------|-----------------|---------------|---------|
| **Archive** | 48 (archive) + 16 (sourcearchive) = 64 | 37 (stage) + 14 (integration) = 51 | GoReleaser slightly ahead |
| **Build** | 37 (pipe/build) + 174 (builders/golang) + ~50 (rust/zig/bun/deno) = ~261 | 12 (stage) + 25 (integration) = 37 | **GoReleaser massively ahead** |
| **Changelog** | 107 | 52 (stage) + 6 (integration) = 58 | **GoReleaser ahead** (2x) |
| **Checksums** | 27 | 33 (stage) + 12 (integration) = 45 | **anodize ahead** |
| **Docker** | 66 (pipe) + docker v2 tests | 25 (stage) + 4 (integration) = 29 | **GoReleaser well ahead** |
| **nFPM** | 75 | 30 (stage) + 5 (integration) = 35 | **GoReleaser ahead** (2x) |
| **Release** | 40 (pipe) + 85 (client tests) = 125 | 48 (stage) + 5 (integration) = 53 | **GoReleaser well ahead** |
| **Sign** | 52 (sign_test) + sign_binary + sign_docker = ~75 | 21 (stage) + 1 (integration) = 22 | **GoReleaser well ahead** (3x) |
| **Announce** | 9 (orchestrator) + ~95 (individual providers) = ~104 | 11 (stage) + 9 (integration) = 20 | **GoReleaser well ahead** (5x) |
| **Homebrew** | 94 | 12 (stage) + ~5 (integration) = 17 | **GoReleaser massively ahead** (5x) |
| **Scoop** | 34 | 11 (stage) + ~3 (integration) = 14 | **GoReleaser ahead** (2.5x) |

### Stages GoReleaser Has That We Don't Test

| Stage | GoReleaser tests | anodize equivalent |
|-------|-----------------|-------------------|
| Snapcraft | 35 | None |
| AUR | 38 | None |
| Chocolatey | 16 | None |
| Flatpak | 18 | None |
| Ko (container) | 45 | None |
| Krew (kubectl plugins) | 39 | None |
| WinGet | 61 | None |
| Nix | 54 | None |
| Blob (S3/GCS/Azure) | 26 | None |
| SBOM | 45 | None |
| UPX (binary compression) | 19 | None |
| Universal Binary (macOS) | 31 | None |
| Makeself (self-extracting) | 9 | None |
| GoMod proxy | 39 | None (N/A for Rust) |
| Notary (macOS signing) | Tests exist | None |
| Artifactory/Upload | Tests exist | None |

This represents **~475+ tests** for stages/features that anodize doesn't have at all.

### Stages Where anodize Is Competitive or Ahead

| Stage | Notes |
|-------|-------|
| **Checksums** | 45 tests vs 27. We test more algorithms (blake2b, blake2s, sha384, sha224) and more edge cases |
| **Config parsing** | 265 tests (every field, valid/default/invalid/edge) vs ~19. We are **dramatically ahead** |
| **Template engine** | 41 tests vs 92. GoReleaser has more template tests, but ours use Tera which is more capable |
| **Custom publishers** | 36 tests (homebrew+scoop+crates_io+lib) is decent coverage for our scope |

---

## 4. Test Categories

### Config Parsing Depth

| Aspect | GoReleaser | anodize |
|--------|-----------|---------|
| Config parsing tests | ~19 (top-level YAML loading, version checks, error cases) | **265** (every field, every type, valid/default/invalid/edge) |
| Per-field validation | Tested implicitly in pipe Default() methods | Tested explicitly with dedicated test per field |
| Config error tests | Scattered across pipes | Centralized + comprehensive |

**Verdict: anodize is dramatically stronger.** GoReleaser's config testing philosophy
is to validate config correctness at the pipe level (each pipe's `Default()` method
validates its section). anodize tests config parsing exhaustively at the config layer.
This is a genuine architectural advantage.

### Behavior Tests (does config produce correct output?)

| Aspect | GoReleaser | anodize |
|--------|-----------|---------|
| Stage behavior tests | ~800+ across all pipes | ~60 |
| Golden file comparisons | Extensive (brew formulas, scoop manifests, nix expressions, krew manifests, winget manifests, AUR PKGBUILDs) | None |

**Verdict: GoReleaser is far ahead.** Golden file testing for generated package
manager manifests is a pattern anodize hasn't adopted. GoReleaser verifies exact
output of generated Homebrew formulas, Scoop manifests, etc. against reference
files. This catches regressions in template rendering that unit tests miss.

### Error Path Tests

| Aspect | GoReleaser | anodize |
|--------|-----------|---------|
| Error path tests | ~200+ (invalid templates, missing env vars, bad configs, failed hooks, etc.) | ~56 |

**Verdict: GoReleaser ahead** by volume, but anodize's error tests are well-structured
and cover the critical paths. GoReleaser's advantage is breadth (more pipes = more
error paths to test).

### E2E / Integration Tests

| Aspect | GoReleaser | anodize |
|--------|-----------|---------|
| Full pipeline tests | cmd/release_test.go (~20), cmd/build_test.go (~31) | 22 E2E pipeline tests |
| Real builds | Yes (Go builds, Rust builds via builders) | Yes (real cargo builds) |
| Real git repos | Yes (via testlib git helpers) | Yes (via TestContextBuilder) |
| Real file I/O | Yes | Yes |
| CI matrix testing | Ubuntu + Windows | Single platform |

**Verdict: Roughly comparable for scope-adjusted comparison.** Both projects test
real builds, real git repos, and real file I/O. GoReleaser tests on multiple
platforms (Ubuntu + Windows). anodize's 22 E2E tests are proportionally reasonable
for its codebase size.

### Fuzz Tests

| Aspect | GoReleaser | anodize |
|--------|-----------|---------|
| Fuzz test functions | 7 (5 template, 2 artifact checksum) | 0 |
| Fuzz infrastructure | Go's built-in `testing.F` | None |

**Verdict: GoReleaser ahead.** anodize has no fuzz testing. This is a gap, especially
for template rendering and config parsing which are parser-heavy code that benefits
from fuzzing.

---

## 5. Testing Patterns

| Pattern | GoReleaser | anodize |
|---------|-----------|---------|
| Table-driven tests | Pervasive (Go idiom, `t.Run` with test case maps/slices) | Some (Rust `#[test]` per case) |
| Golden files | Yes (`internal/golden` package, RequireEqualRb/Yaml/JSON/Txt) | No |
| Mock clients | `client.NewMock()` tracking method calls | `MockGitHubClient` with expectations |
| httptest servers | Extensive (GitHub, GitLab, Gitea API simulation) | Not used (mock trait instead) |
| Test context builder | `testctx.Wrap()` / `testctx.WrapWithCfg()` with option funcs | `TestContextBuilder` with builder pattern |
| Temp dir helpers | `testlib.Mktmp(t)`, `t.TempDir()` | `tempdir` via TestContextBuilder |
| Git test helpers | `testlib` package (14 files: git.go, archive.go, docker.go, etc.) | `test_helpers.rs` (git repo setup, binary creation) |
| Fuzz testing | 7 fuzz functions across 2 files | None |
| Race detection | `go test -race` flag on every CI run | Not applicable (Rust's ownership model) |
| Cross-platform CI | Ubuntu + Windows matrix | Single platform |
| Test fixtures/testdata | Extensive `testdata/` directories with YAML, JSON, golden files | Inline TOML strings in tests |

### Key Pattern Differences

**GoReleaser's httptest approach vs anodize's mock trait approach:**
GoReleaser spins up real HTTP servers (`httptest.NewServer`) with custom handlers
to simulate GitHub/GitLab/Gitea APIs. This tests real HTTP serialization,
headers, status codes. anodize uses a `MockGitHubClient` trait implementation,
which is simpler but doesn't test HTTP-level concerns.

**Golden files:** GoReleaser's golden file pattern is a significant testing advantage.
When brew/scoop/nix/winget/AUR templates change, tests automatically catch output
regressions by diffing against reference files. anodize has no equivalent.

---

## 6. What GoReleaser Tests That We Don't

### Features We Don't Have (and their test counts)

| Feature | GoReleaser tests | Gap type |
|---------|-----------------|----------|
| Snapcraft packaging | 35 | Feature gap |
| AUR packages | 38 | Feature gap |
| Chocolatey packages | 16 | Feature gap |
| Flatpak packages | 18 | Feature gap |
| Ko container builds | 45 | Feature gap |
| Krew kubectl plugins | 39 | Feature gap |
| WinGet manifests | 61 | Feature gap |
| Nix packages | 54 | Feature gap |
| Cloud blob storage (S3/GCS/Azure) | 26 | Feature gap |
| SBOM generation | 45 | Feature gap |
| UPX compression | 19 | Feature gap |
| macOS Universal Binary | 31 | Feature gap |
| Makeself installers | 9 | Feature gap |
| GoMod proxy | 39 | N/A (Go-specific) |
| macOS notarization | Tests exist | Feature gap |
| Multiple SCM backends (GitLab, Gitea) | ~100 | Feature gap |
| Docker manifest lists | ~20 | Feature gap |
| Milestone management | 19 | Feature gap |
| **Subtotal** | **~615** | |

### Testing Patterns We Don't Use

| Pattern | Impact |
|---------|--------|
| Golden file testing | High - catches template output regressions |
| Fuzz testing | Medium - catches edge cases in parsing |
| httptest HTTP server mocks | Medium - tests real HTTP serialization |
| Cross-platform CI | Medium - catches Windows-specific issues |
| Multiple SCM provider testing | Low for now (only GitHub supported) |

### Specific Test Gaps Within Shared Features

| Area | What GoReleaser tests that we don't |
|------|-------------------------------------|
| **Build** | Multiple builder backends (Go, Rust, Zig, Bun, Deno, Poetry, UV), cross-compilation targets, build hooks (pre/post), build output types, go build lines, ldflags templates. We have 37 tests vs their ~261 |
| **Sign** | Docker image signing, binary signing, multiple signature types, GPG config from git. We have 22 tests vs their ~75 |
| **Changelog** | They test 107 cases including git log parsing, commit grouping, filtering, sorting. We have 58 |
| **Homebrew** | They test 94 cases including golden file formula verification, PR creation, fork syncing, multi-arch. We have 17 |
| **Docker** | They test docker v2 heuristics, digest handling, manifest creation, imager validation. We have 29 |
| **Release** | They test release body generation, SCM-specific release logic, draft/prerelease handling extensively. We have 53 |

---

## 7. What We Test That GoReleaser Doesn't

| Area | anodize advantage | Details |
|------|-------------------|---------|
| **Config parsing exhaustiveness** | Major | 265 tests covering every field individually. GoReleaser has ~19 config-level tests and relies on pipes to validate their own sections. Our approach catches config regressions earlier |
| **Checksum algorithm breadth** | Minor | We test blake2b, blake2s, sha224, sha384 in addition to the common ones |
| **Workspace/multi-crate** | Unique | E2E tests for Cargo workspace dependency ordering, change detection, force flags. GoReleaser's monorepo support is different (single binary project) |
| **Crates.io publishing** | Unique | Tests for crates.io dry-run publishing (GoReleaser has no Rust ecosystem publishing) |
| **TOML config format** | Unique | GoReleaser uses YAML; our TOML parsing is extensively tested |
| **Tera template engine** | Different | We test Tera-specific features; they test Go text/template features |

---

## 8. Honest Summary

### The Numbers

- **GoReleaser: ~2,000 tests across 164 files** with 7 fuzz tests, golden file testing,
  httptest servers, cross-platform CI, and coverage via Codecov.
- **anodize: 814 tests across ~30 modules** with 80.4% line coverage, strong config
  parsing tests, and solid E2E pipeline tests.

### Where GoReleaser Is Stronger

1. **Raw test volume:** 2-3x more test cases, covering far more feature surface area
2. **Build stage testing:** ~261 tests vs our 37. This is the biggest gap in shared features
3. **Golden file testing:** They verify exact template output. We don't do this at all
4. **Announcement providers:** ~104 tests vs our 20. Each provider is individually tested
5. **Package manager publishers:** Brew (94), Scoop (34), plus 8 more we don't have
6. **Signing:** 75 tests with docker/binary/GPG variants vs our 22
7. **SCM client testing:** httptest servers simulating GitHub/GitLab/Gitea APIs
8. **Fuzz testing:** 7 fuzz functions vs zero
9. **Cross-platform CI:** They test on Windows. We don't
10. **Feature breadth:** ~615 tests for features we don't have at all

### Where anodize Is Stronger

1. **Config parsing:** 265 tests vs ~19. This is our clear win
2. **Checksums:** 45 tests vs 27, with more algorithm coverage
3. **Coverage transparency:** 80.4% is verified and published
4. **Workspace support testing:** Unique to our Cargo-native approach

### Where We're Roughly Equal

1. **E2E pipeline tests:** Both test real builds, git repos, file I/O
2. **Test infrastructure:** Both have context builders, mock clients, temp dir helpers
3. **Template testing:** Both extensively test their template engines (92 vs 41 tests)

### Bottom Line

**GoReleaser's test suite is substantially larger and more mature.** This is expected
for a project that is 8+ years old with 450+ contributors vs a new project. The gap
is mostly driven by:

1. Feature breadth (they have 60+ pipes vs our ~12 stages)
2. More tests per shared feature (especially build, sign, changelog, brew)
3. Testing patterns we haven't adopted (golden files, fuzz, httptest)

**However, anodize's test quality is strong for its scope.** 814 tests at 80.4%
coverage with 265 config parsing tests is solid engineering. The config parsing
depth is genuinely better than GoReleaser's approach. The E2E tests are
proportionally competitive.

**Priority gaps to close (highest impact first):**

1. **Build stage tests** (37 vs 261) - This is the most important stage and our weakest
2. **Golden file testing** for generated Homebrew formulas and Scoop manifests
3. **Sign stage tests** (22 vs 75) - Critical for release trust
4. **Changelog tests** (58 vs 107) - Important for user-facing output
5. **Fuzz tests** for template engine and config parser
6. **Homebrew/Scoop publisher tests** (17/14 vs 94/34) - Primary distribution channels
7. **Release stage coverage** (51% line coverage is the weakest stage)
8. **Announce stage coverage** (18-38% is concerning)

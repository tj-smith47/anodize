# Root cause: cfgd v0.3.5 krew PR #5633 CI failure

Date: 2026-04-25
Tracked task: #24 (Group 2.1)

## Confirmed root cause

`crates/stage-publish/src/krew.rs::artifacts_to_platforms` (lines 184-210) and the empty-artifacts fallback at lines 371-380 set `bin: <crate_name>` for every platform, with no `.exe` suffix logic for Windows. Windows zips contain `cfgd.exe` (and `anodizer.exe`), so the krew validator and real installs cannot find the binary literal `cfgd` after extraction.

CI failure (PR #5633):
```
F0420 04:54:23.593348 main.go:63] spec.platforms[0] failed to install: plugin install command failed
W0420 04:54:23.589418 install.go:164] failed to install plugin "cfgd": ... can't create symbolic link, source binary ("/tmp/krew-test.../store/cfgd/v0.3.5/cfgd") cannot be found in extracted archive
```

## Krew validator behavior (confirmed from source)

`kubernetes-sigs/krew/cmd/validate-krew-manifest/main.go` + `internal/installation/install.go`:
- The validator iterates `spec.platforms[i]` in manifest order, runs `kubectl krew install` per platform with `KREW_OS`/`KREW_ARCH` overrides — `platforms[0]` is literally the first entry in the YAML (cfgd's case: `windows-arm64`).
- `install()` does `filepath.Join(installDir, filepath.FromSlash(op.platform.Bin))` then symlinks. The `Bin` field is taken **literally** — krew does not append `.exe`. So on Windows, `bin:` MUST include `.exe` if that's how the binary is named in the zip.

## GoReleaser behavior (`/opt/repos/goreleaser/internal/pipe/krew/krew.go:233-247`)

GR uses `bins[0]` from the archive's `ExtraBinaries` extra, which goreleaser populates from `binary.Name`. The Go builder appends `.exe` to `binary.Name` for windows targets (`internal/builders/golang/build_test.go:600,735,750`). So GR produces `Bin: "cfgd.exe"` for windows platforms naturally — no explicit `if windows { append .exe }` in krew.go because the upstream builder did it.

## Anodizer-side gap

Anodizer doesn't track per-archive binary filenames the way goreleaser does (its `OsArtifact` has os/arch/url/sha256 but not the in-archive binary name), so `artifacts_to_platforms` blanket-uses `crate_name` and never appends `.exe`.

## Minimal fix

`crates/stage-publish/src/krew.rs::artifacts_to_platforms`:

```rust
fn artifacts_to_platforms(artifacts: &[OsArtifact], binary_name: &str) -> Vec<KrewPlatform> {
    let mut platforms = Vec::new();
    for a in artifacts {
        let os = krew_os(&a.os).to_string();
        let bin = if os == "windows" {
            format!("{binary_name}.exe")
        } else {
            binary_name.to_string()
        };
        // ... use `bin.clone()` / `bin` in both arch == "all" expansion (line ~196)
        // and the regular branch (line ~205) instead of `binary_name.to_string()`
    }
}
```

The empty-artifacts placeholder at krew.rs:371-380 is linux-only so `bin: crate_name` is correct there; no change needed.

**Test plan:** unit test that builds platforms from a windows + linux `OsArtifact` pair and asserts `bin == "cfgd.exe"` on the windows entry and `bin == "cfgd"` on linux.

## Per-plan deep-dive scope (Task 2.1)

Before pushing to the fork branch, also audit:
- `bin` path (.exe on windows) — covered above
- `files:` field for archives with subfolders (krew docs allow non-flat archives)
- Caveats / shortDescription length limits / line-wrap (krew best-practices linter)
- Platform selector completeness — should we skip windows-arm64 if no asset exists?
- darwin-universal handling (we expand all → amd64+arm64; verify still correct after .exe change)
- url_template support
- sha256 — verify still correctly populated after stage-checksum's `Checksum`-vs-`sha256` key fix (commit fede6499)
- Anything else flagged in `publishers-pkgmgr.md`:
  - GR krew.go:86-91 errors on empty `description`/`short_description` — verify anodizer replicates
  - KrewConfig has anodizer-only `disable` field (Group 1.2 removes)
  - Default commit_msg `"Krew manifest update for ..."`

## Reference files

- `/opt/repos/anodizer/crates/stage-publish/src/krew.rs` (lines 184-210, 371-389)
- `/opt/repos/goreleaser/internal/pipe/krew/krew.go` (lines 232-247)
- `/opt/repos/goreleaser/internal/pipe/archive/archive.go` (lines 225-248)
- krew validator + installer: `kubernetes-sigs/krew` `cmd/validate-krew-manifest/main.go` + `internal/installation/install.go`

## Manual-push procedure (for after fix is verified locally)

1. Apply the .exe fix + any other gaps surfaced in the deep-dive.
2. Locally regenerate the cfgd v0.3.5 krew manifest (investigate `--only=krew` or run anodizer with selective stage flags). Resulting `plugins/cfgd.yaml` should change ONLY in the windows-arm64 and windows-amd64 entries (`bin: cfgd` → `bin: cfgd.exe`), unless other gaps require additional changes.
3. Verify regenerated manifest: SHAs match the v0.3.5 GitHub release, archive layouts match, all six platforms present, windows bins have `.exe`.
4. Present manifest preview to user. STOP for explicit per-push approval.
5. After user OK: clone tj-smith47/krew-index fork branch `cfgd-v0.3.5`, replace `plugins/cfgd.yaml`, commit (`task commit -- ...` if applicable; or `git -c user.email=tj@jarvispro.io commit ...` per memory note about github email), push.
6. PR #5633 auto-updates, CI re-runs.

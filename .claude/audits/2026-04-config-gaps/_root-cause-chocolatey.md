# Root cause: anodizer + cfgd missing from Chocolatey community gallery

Date: 2026-04-25
Tracked task: #25 (Group 2.2)

## Confirmed root cause

Both anodizer and cfgd packages **were successfully pushed to Chocolatey** but are **stuck in the community moderation queue** — `<d:PackageStatus>Submitted</d:PackageStatus>` and `<d:Published>1900-01-01T00:00:00</d:Published>`. New packages on community.chocolatey.org require human moderator approval before becoming "Listed" and appearing in `?$filter=Id eq '...'` queries or the gallery UI. Until then, the OData direct-key endpoint `Packages(Id='X',Version='Y')` returns the entry, but the public listing endpoint hides it.

## Compounding bug (the latent silent skip)

`crates/stage-publish/src/chocolatey.rs:587-595` treats `FeedHashResult::Present { hash matches }` as "already on feed → skipping" and returns `Ok(())` without checking listing/moderation state. Result: every subsequent re-release silently no-ops. Green CI, no gallery presence, no error.

Sequence per release:
- First push attempts: HTTP 403 from Cloudflare/IIS edge (transient false-positive). The bash-level retry wrapper hides this.
- Eventually one attempt returns 201 (push succeeded → moderation queue).
- Subsequent runs hit `package_feed_hash() → Present`, hash matches our locally-computed hash, publisher prints "skipping (hash match)" and returns Ok.

## Specific fix sites (all in `crates/stage-publish/src/chocolatey.rs`)

1. **Lines 587-624** (the match on `FeedHashResult::Present`):
   - New decision table:
     - `Present { listed: true, hash matches }` → log "already published — skipping" and return Ok.
     - `Present { listed: true, hash differs }` → bail with existing immutability error.
     - `Present { listed: false, status="Submitted"|"Unknown"|"Rejected"|"Exempted" }` → log clear status message ("X 0.0.0 is in Chocolatey moderation as Submitted since YYYY-MM-DD; not re-pushing") and return Ok without re-pushing (Chocolatey rejects re-pushes of submitted versions). For `Rejected`, surface as a hard error if rejection reason is available.
     - `Absent` → push as today.

2. **Lines 767-814** (`package_feed_hash`):
   - Parse `<d:PackageStatus>` and `<d:Listed>` (and/or `<d:Published>`; `1900-01-01` is the unlisted sentinel) in addition to `PackageHash`/`PackageHashAlgorithm`.
   - Return them in `FeedHashResult::Present { hash, algorithm, status, listed, published }`.

3. **Lines 442-456** (the `(None, None)` fallback in `publish_to_chocolatey`):
   - Currently pushes a nupkg whose install script has empty checksum and a fabricated GitHub URL. **This is likely the original cause of moderation accumulation** — moderators reject this as broken.
   - Replace with hard error.

4. **Line 922** (`push_nupkg` success check):
   - HTTP 403 with HTML body (Cloudflare/IIS challenge) is transient. Detect Content-Type: text/html plus 403/503 status; retry once internally OR include clearer hint in error so bash-level retry isn't the only safety net. Lower priority.

## Operator helper to add (separate task)

A `--chocolatey-status` (or `anodize doctor chocolatey`) command that calls `package_feed_hash` and reports `PackageStatus`/`Published`/`Listed` for configured packages, so users can see queue state without grepping CI logs.

## GoReleaser parity notes (don't act on these without separate review)

- GR declares hard `Dependencies()` requirement on the `choco` binary (chocolatey.go:40); ours implements native nupkg packing + raw NuGet V2 PUT. Deliberate divergence (lets us run on Linux runners). Document so users inheriting a GR `chocolatey:` block aren't surprised.
- GR maps `goamd64=v1` (default) explicitly (chocolatey.go:99-108); we apply `amd64_variant.or(Some("v1"))` at chocolatey.rs:326 with permissive filter (`is_none_or(|v| v == want)`) at 351-360. GR's filter excludes artifacts whose `goamd64` doesn't match.
- GR fails publish loudly when no Windows archive exists (errNoWindowsArchive at chocolatey.go:21,120). Ours warns and pushes a placeholder. **This is the most consequential parity gap and likely the original cause of broken moderation submissions** — covered by fix site #3 above.
- GR's `doPush` at chocolatey.go:189-193: when `key == ""` after templating, log warn + return nil. Ours does same at chocolatey.rs:565-571. Parity.
- GR has no "already on feed → skip" idempotency. Our drift-detection is anodizer-only. Fix above keeps the feature but makes it correct.

## Verification (per plan)

- Mock OData test cases for each `FeedHashResult::Present` variant (listed=true/false × hash-match/mismatch × status=Submitted/Listed/Rejected).
- Integration test: no-windows-artifact returns `Err`, not `Ok`.
- Production verification once next release runs and CI logs show "in moderation as Submitted since ..." instead of silent "hash match skip".

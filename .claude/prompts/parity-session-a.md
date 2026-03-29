# Parity Session A: All Publishers — Config Field Parity

## Your Task

Implement every unchecked item under **Session A** in `.claude/specs/parity-session-index.md`. Read that file first — it has the parity definition, session rules, and the full checklist.

## Session Rules (from the index)

1. **Read GoReleaser source first.** Before implementing ANY feature, read the Go source at `/opt/repos/goreleaser/internal/pipe/{area}/` and its tests. List every config field, default, and behavior.
2. **Spec + code quality review loop.** After implementing, run spec review then code quality review. Fix ALL findings of ANY severity. Re-review. Repeat until ZERO issues/suggestions remain.
3. **Mark items done.** Check the box in `parity-session-index.md` when implemented.
4. **Work on master directly.** No worktrees or branches.

## Scope

Session A covers config field parity for all 7 publishers plus the shared repository/commit_author structs:

- **Shared repository config** — implement once, use everywhere: branch, token, token_type, pull_request.* (enabled, draft, check_boxes, body, base.*), git.* (url, private_key, ssh_command), commit_author.signing.* (enabled, key, program, format)
- **Homebrew** — ids, url_template, url_headers, download_strategy, custom_require, custom_block, extra_install, post_install, plist, service, Homebrew Casks (entire feature), alternative_names, app, PR config
- **Scoop** — url_template, use (archive/msi/nsis), 32-bit architecture block
- **Chocolatey** — ids, owners, title, copyright, require_license_acceptance, project_source_url, docs_url, bug_tracker_url, summary, release_notes, dependencies, source_repo, use, url_template, package_source_url
- **Winget** — ids, use, product_code, url_template, commit_msg_template, path, homepage, license_url, copyright, copyright_url, skip_upload, release_notes, release_notes_url, installation_notes, tags, dependencies, publisher_support_url, privacy_url, repository.*, commit_author.*
- **AUR** — ids, private_key, skip_upload, commit_msg_template, git_ssh_command
- **Krew** — ids, url_template, commit_msg_template, skip_upload, repository.*, commit_author.*
- **Nix** — full new publisher: name, path, install, extra_install, post_install, dependencies, formatter, repository.*, commit_author.*

## Approach

1. Start with the **shared repository config struct** — this is used by all 7 publishers, so get it right first.
2. Then work through each publisher alphabetically, reading the GoReleaser source for each one before implementing.
3. For each publisher, ensure config fields are not just parsed but **wired through to behavior** (the formula/manifest/PKGBUILD generation must actually use them).
4. After each publisher, review and fix before moving to the next.

## GoReleaser Source Locations

- Homebrew: `/opt/repos/goreleaser/internal/pipe/brew/`
- Scoop: `/opt/repos/goreleaser/internal/pipe/scoop/`
- Chocolatey: `/opt/repos/goreleaser/internal/pipe/chocolatey/`
- Winget: `/opt/repos/goreleaser/internal/pipe/winget/`
- AUR: `/opt/repos/goreleaser/internal/pipe/aur/`
- Krew: `/opt/repos/goreleaser/internal/pipe/krew/`
- Nix: `/opt/repos/goreleaser/internal/pipe/nix/`

## Do NOT

- Implement items from other sessions (B, C, D, E, F)
- Skip any item as "niche" or "low priority"
- Trust summaries — read actual Go source per publisher
- Add config fields without wiring them to behavior

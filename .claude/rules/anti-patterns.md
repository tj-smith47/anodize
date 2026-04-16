---
paths: ["**/*.rs"]
---
# anodize anti-patterns (hook-enforced)

The project-level `post-edit.sh` hook enforces these. Fix violations immediately —
the hook blocks the next turn until you do.

## In any `.rs` file (except `tests/**`, `*_test.rs`, `*/main.rs`)

| Pattern | Rule | Replacement |
|---|---|---|
| `.unwrap()` / `.expect(` | Quality | `?` with proper error type |
| `use log::*` | Style | `use tracing::*` |
| `#[allow(dead_code)]` | Hygiene | Delete the unused code |

## Security

| Pattern | Rule | Action |
|---|---|---|
| `github_pat_[A-Za-z0-9_]{20,}` | Hardcoded credential | Use secret resolver |
| `ghp_[A-Za-z0-9]{36}` | Hardcoded GitHub token | Use secret resolver |

## Verification
Run `/verify` (cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test).

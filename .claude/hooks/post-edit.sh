#!/usr/bin/env bash
# anodize project post-edit hook: Rust conventions for a release tool.
# Scoping (honest):
#   - BLOCK (exit 2): things that MUST be immediately correct (hardcoded secrets).
#   - WARN  (exit 0, stderr): convention drift tracked as migration in known-bugs.md.
#     Per-edit blocking on migration-scope issues just creates noise we ignore.
set -euo pipefail

path=$(jq -r '.tool_input.file_path // empty' 2>/dev/null)
[[ -z "$path" || ! -f "$path" || "$path" != *.rs ]] && exit 0

blockers=()
warns=()
check_blocker() {
  if grep -Eq "$1" "$path" 2>/dev/null; then blockers+=("$2"); fi
}
check_warn() {
  if grep -Eq "$1" "$path" 2>/dev/null; then warns+=("$2"); fi
}

# BLOCKERS — always exit 2. Real safety issues.
check_blocker 'github_pat_[A-Za-z0-9_]{20,}' 'Hardcoded GitHub PAT detected — use secret resolver'
check_blocker 'ghp_[A-Za-z0-9]{36}'          'Hardcoded GitHub classic token detected'

# WARNS — advisory, tracked in .claude/known-bugs.md as migration scopes.
if [[ "$path" != */main.rs && "$path" != */tests/* && "$path" != *_test.rs ]]; then
  if grep -nE '\.unwrap\(\)|\.expect\(' "$path" 2>/dev/null | grep -vE '#\[test\]|#\[cfg\(test\)\]|mod tests' >/dev/null; then
    warns+=('unwrap/expect in non-test code — tracked in known-bugs for migration to ? + Context')
  fi
fi
check_warn '^use log::'                  'log::* should be tracing::* — tracked migration'
check_warn '#\[allow\(dead_code\)\]'     'dead_code suppression — delete or justify'

if (( ${#warns[@]} > 0 )); then
  printf 'anodize advisory in %s:\n' "$path" >&2
  printf '  - %s\n' "${warns[@]}" >&2
fi
if (( ${#blockers[@]} > 0 )); then
  printf 'anodize BLOCKERS in %s:\n' "$path" >&2
  printf '  - %s\n' "${blockers[@]}" >&2
  exit 2
fi
exit 0

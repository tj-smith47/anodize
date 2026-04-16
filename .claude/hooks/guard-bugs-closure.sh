#!/usr/bin/env bash
# PreToolUse guard for .claude/known-bugs.md closures.
#
# Catches the "lazy write-off" pattern: marking a bug as resolved (`[ ]`→`[x]`
# or adding a new `[x]` line) without any corresponding source change in the
# session, and without citing verifiable evidence.
#
# Fires on Edit/Write whose target is `.claude/known-bugs.md` AND whose diff
# introduces at least one new `- [x]` line.
#
# Allow paths (any one is sufficient):
#   1. Session has staged/unstaged/untracked/recent-commit changes whose
#      paths or backticked identifiers appear in the closed entry body.
#   2. The closed entry body contains `audit: <citation>` where the citation
#      resolves to an existing file path or a discoverable source identifier.
#   3. The closed entry body contains `AUDITED: <reason>` (explicit in-band
#      override — reviewable in git blame by design).
#
# Otherwise: exit 2 with an explanation. No env-var bypass — every skip must
# leave a git-blameable trail.
set -uo pipefail

input=$(cat)
tool_name=$(printf '%s' "$input" | jq -r '.tool_name // ""')
file_path=$(printf '%s' "$input" | jq -r '.tool_input.file_path // ""')

# Scope: only `.claude/known-bugs.md` under any repo.
case "$file_path" in
  */.claude/known-bugs.md) ;;
  *) exit 0 ;;
esac

old=""
new=""
case "$tool_name" in
  Edit)
    old=$(printf '%s' "$input" | jq -r '.tool_input.old_string // ""')
    new=$(printf '%s' "$input" | jq -r '.tool_input.new_string // ""')
    ;;
  Write)
    new=$(printf '%s' "$input" | jq -r '.tool_input.content // ""')
    [ -f "$file_path" ] && old=$(cat -- "$file_path")
    ;;
  *) exit 0 ;;
esac

# Extract all `- [x]` lines from old and new.
old_checked=$(printf '%s\n' "$old" | grep -E '^[[:space:]]*-[[:space:]]*\[x\][[:space:]]' 2>/dev/null | sort -u)
new_checked=$(printf '%s\n' "$new" | grep -E '^[[:space:]]*-[[:space:]]*\[x\][[:space:]]' 2>/dev/null | sort -u)

# Newly-closed lines = those in new but not old. comm requires sorted input.
added_closures=$(
  comm -13 \
    <(printf '%s\n' "$old_checked") \
    <(printf '%s\n' "$new_checked") \
    | grep -vE '^[[:space:]]*$' || true
)

if [ -z "$added_closures" ]; then
  exit 0
fi

# For hint extraction, use the full `new_string`. If the user's edit is a
# multi-line entry, it's all there. If the edit is a `[ ]`→`[x]` toggle on a
# continuation-heavy entry, the toggle itself contains the relevant signals.
entry_text="$new"

# Allow path #3: AUDITED: <reason> in-band bypass.
if printf '%s\n' "$entry_text" | grep -qE 'AUDITED:[[:space:]]+[^[:space:]]'; then
  exit 0
fi

# Resolve repo root from file_path: .../repo/.claude/known-bugs.md → .../repo
repo_root=$(cd -- "$(dirname -- "$file_path")/.." 2>/dev/null && pwd)
if [ -z "$repo_root" ] || [ ! -d "$repo_root/.git" ]; then
  # Not in a git repo — skip, can't verify.
  exit 0
fi

# Allow path #2: `audit: <citation>` — must start its own line (leading
# whitespace allowed). This avoids matching words ending in "audit:" like
# `parity-audit:` which is a common entry prefix in this repo.
audit_citations=$(
  printf '%s\n' "$entry_text" \
    | grep -iE '^[[:space:]]*audit:[[:space:]]+[^[:space:]]+' \
    | sed -E 's/^[[:space:]]*[Aa][Uu][Dd][Ii][Tt]:[[:space:]]+//' \
    | awk '{print $1}'
)
if [ -n "$audit_citations" ]; then
  all_good=1
  while IFS= read -r cite; do
    [ -z "$cite" ] && continue
    # Strip :line suffix if present.
    path_part=${cite%%:*}
    if [ -e "$repo_root/$path_part" ]; then
      continue
    fi
    # Maybe it's a test name / identifier — grep the tree.
    if (cd "$repo_root" && grep -rqF --include='*.rs' -- "$cite" crates 2>/dev/null); then
      continue
    fi
    all_good=0
    break
  done <<EOF
$audit_citations
EOF
  if [ "$all_good" = "1" ]; then
    exit 0
  fi
fi

# Allow path #1: session diff overlap.
# Session paths: unstaged + staged + untracked + recent (2h) commits.
session_paths=$(
  cd "$repo_root" 2>/dev/null && {
    git diff --name-only HEAD 2>/dev/null
    git diff --name-only --cached 2>/dev/null
    git ls-files --others --exclude-standard 2>/dev/null
    git log --since='2 hours ago' --name-only --pretty=format: 2>/dev/null
  } | sort -u | grep -vE '^$'
)

# Hints from entry body: backticked identifiers + file-path-looking tokens.
# Anything that looks like a path, module, or backticked name.
bt_hints=$(
  printf '%s\n' "$entry_text" \
    | grep -oE '`[^`]+`' \
    | sed -E 's/`//g' \
    | sort -u
)
path_hints=$(
  printf '%s\n' "$entry_text" \
    | grep -oE '[A-Za-z0-9_./-]+\.(rs|toml|yaml|yml|sh|md)\b' \
    | sort -u
)
hints=$(printf '%s\n%s\n' "$bt_hints" "$path_hints" | sort -u | grep -vE '^$' || true)

overlap=0
if [ -n "$session_paths" ] && [ -n "$hints" ]; then
  # Also dump the full session diff once for identifier lookups.
  session_diff=$(cd "$repo_root" 2>/dev/null && git diff HEAD 2>/dev/null)
  while IFS= read -r hint; do
    [ -z "$hint" ] && continue
    # Path substring match against changed file list.
    if printf '%s\n' "$session_paths" | grep -Fq -- "$hint"; then
      overlap=1
      break
    fi
    # Identifier substring match against diff content.
    if [ -n "$session_diff" ] && printf '%s\n' "$session_diff" | grep -Fq -- "$hint"; then
      overlap=1
      break
    fi
  done <<EOF
$hints
EOF
fi

if [ "$overlap" = "1" ]; then
  exit 0
fi

# No allow path matched — block.
cat >&2 <<EOF
BLOCKED: closing a known-bugs entry without verifiable proof of work.

Closed entry line(s):
$(printf '%s\n' "$added_closures" | sed 's/^/  /')

Checked, none matched:
  - session changes (uncommitted + staged + untracked + 2h commits)
    contained no path or backticked identifier from the entry.
  - no valid "audit: <citation>" lines found in the entry body.
  - no "AUDITED: <reason>" in-band bypass present.

This is the lazy-write-off guard. Options to proceed:

  1. Prior-session fix: append a line to the resolved entry:
         audit: crates/cli/src/pipeline.rs
         audit: test_detect_nfpm_builds_under_crates
     The hook requires each citation to resolve (as a path or an
     identifier grep'd from crates/). Cite the minimum.

  2. Claim was wrong / no fix needed: read the relevant code path first,
     then close with "AUDITED: <one-line reason>" on the entry line.

  3. This-session work: write the code + tests first, then close the box.
EOF
exit 2

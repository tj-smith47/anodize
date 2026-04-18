#!/usr/bin/env bash
# Parity-loop preflight — emits sentinel + wave + bug counts for /parity-loop
# dispatch. No side effects (read-only). Single allowlist entry covers every tick.
#
# Output keys: sentinel, a (anodize+anodize-action wave-A unchecked),
#              b (cfgd wave-B unchecked), c (anodize wave-C unchecked),
#              bugs (total unchecked across 3 known-bugs.md).
set -euo pipefail

SENTINEL=/opt/repos/anodize/.claude/audits/2026-04-v0.x/PARITY-ACHIEVED
if [ -f "$SENTINEL" ]; then
  echo "sentinel=yes"
else
  echo "sentinel=no"
fi

# grep returns non-zero on zero matches; pipefail would abort the script.
a_raw=$(grep -hE '^- \[ \] A[0-9]+ —' \
  /opt/repos/anodize/.claude/audits/2026-04-v0.x/wave-status.md \
  /opt/repos/anodize-action/.claude/audits/2026-04-v0.x/wave-status.md 2>/dev/null || true)
a=$(echo -n "$a_raw" | grep -c '^' || true)
a=${a:-0}
# grep -c exits 1 when zero matches under `set -e`; wrap to tolerate.
b=$(grep -cE '^- \[ \] B[0-9]+ —' \
  /opt/repos/cfgd/.claude/audits/2026-04-v0.x/wave-status.md 2>/dev/null || true)
b=${b:-0}
c=$(grep -cE '^- \[ \] C[0-9]+ —' \
  /opt/repos/anodize/.claude/audits/2026-04-v0.x/wave-status.md 2>/dev/null || true)
c=${c:-0}

total=0
for f in /opt/repos/anodize/.claude/known-bugs.md \
         /opt/repos/anodize-action/.claude/known-bugs.md \
         /opt/repos/cfgd/.claude/known-bugs.md; do
  [ -f "$f" ] || continue
  n=$(grep -c '^- \[ \]' "$f" || true)
  total=$((total + n))
done

echo "a=$a b=$b c=$c bugs=$total"

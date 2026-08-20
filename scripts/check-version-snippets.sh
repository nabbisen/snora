#!/usr/bin/env bash
set -euo pipefail

# RFC-074: finds every `snora`/`snora-*` version-bearing Cargo snippet under
# docs/ and crate doc comments and reports any whose minor does not match the
# current release. The expected minor is derived from
# [workspace.package].version in Cargo.toml (RFC-074 Q-2), not hard-coded —
# a check carrying its own expected version would be this exact defect one
# level down.
#
# Not wired into CI (RFC-074 Q-1) — run manually before a release cut, same
# shape and reasoning as scripts/check-built-links.py (RFC-073).
#
# Usage: scripts/check-version-snippets.sh

cd "$(git rev-parse --show-toplevel)"

CURRENT_MINOR=$(grep -m1 '^version = "[0-9]\+\.[0-9]\+\.[0-9]\+"' Cargo.toml \
  | sed -E 's/.*"([0-9]+\.[0-9]+)\.[0-9]+".*/\1/')
if [[ -z "$CURRENT_MINOR" ]]; then
  echo "error: could not derive expected minor from [workspace.package].version in Cargo.toml" >&2
  exit 1
fi
echo "Expected minor (from Cargo.toml [workspace.package].version): $CURRENT_MINOR"
echo

# Two Cargo snippet shapes appear in this repo: bare `snora = "X.Y[.Z]"` and
# table form `snora = { version = "X.Y[.Z]", ... }`. Package name is `snora`
# or `snora-<kebab>` (e.g. snora-design, snora-widgets, snora-style).
SNIPPET_RE='snora(-[a-z][a-z0-9-]*)?[[:space:]]*=[[:space:]]*(\{[^}]*version[[:space:]]*=[[:space:]]*)?"[0-9]+\.[0-9]+(\.[0-9]+)?"'

stale=0
checked=0

# Exclusion set (RFC-074 §4) — a check that cannot tell a live snippet from a
# quoted one is worse than no check, since it will either cry wolf every
# release or get silenced. Pruned from the walk entirely:
#   docs/book/**    — generated output, regenerated from docs/src on every
#                      build; never the source of a finding.
#   .git-exclude/**  — review-request/handoff archive, not published
#                      documentation.
#   rfcs/**          — an RFC may quote a stale version as the finding it is
#                      reporting (RFC-051 quotes a consumer's then-current
#                      "0.25"; RFC-056 quotes "snora-widgets = \"0.6\"" as the
#                      stale instruction it was reporting). Rewriting either
#                      would destroy the finding.
#   target/, .git/   — build output / VCS internals.
while IFS= read -r -d '' file; do
  rel="${file#./}"

  case "$rel" in
    # docs/src/guides/migration-*.md — these document what a version *was*
    # (RFC-069 Q-2: deliberately historical, staleness is the content).
    docs/src/guides/migration-*.md) continue ;;
    # CHANGELOG.md — same: a historical record, not current guidance.
    CHANGELOG.md) continue ;;
  esac

  while IFS=: read -r lineno match; do
    [[ -z "${lineno:-}" ]] && continue
    checked=$((checked + 1))
    ver=$(grep -oE '"[0-9]+\.[0-9]+(\.[0-9]+)?"' <<<"$match" | head -1 | tr -d '"')
    minor=$(cut -d. -f1,2 <<<"$ver")
    if [[ "$minor" != "$CURRENT_MINOR" ]]; then
      echo "STALE  $rel:$lineno  found \"$ver\", expected minor \"$CURRENT_MINOR\""
      stale=$((stale + 1))
    fi
  done < <(grep -nEo "$SNIPPET_RE" "$file" || true)
done < <(find . -type d \( -name .git -o -name target -o -path ./docs/book -o -path ./.git-exclude -o -path ./rfcs \) -prune \
  -o -type f \( -name '*.md' -o -name '*.rs' \) -print0)

echo
echo "Checked $checked version-bearing snippet(s)."
if [[ $stale -gt 0 ]]; then
  echo "$stale stale snippet(s) found." >&2
  exit 1
fi
echo "All current."

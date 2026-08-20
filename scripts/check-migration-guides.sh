#!/usr/bin/env bash
set -euo pipefail

# RFC-079: for every consecutive minor release, confirms a migration guide
# exists. `docs/src/guides/migrations.md` promises "each minor release
# ships a focused migration guide" — unconditionally, even to say nothing
# is required. This derives every consecutive minor pair from `git tag`
# and the filesystem rather than a hand-maintained list, and reports every
# gap it finds.
#
# ADOPTION_MINOR is the one boundary constant (RFC-079 Q-3): gaps at or
# after this minor fail the check; earlier gaps are reported as known
# historical and do not fail it. RFC-079 named five explicitly as
# deferred by the owner (0.29->0.30, 0.30->0.31, 0.31->0.32, 0.34->0.35,
# 0.37->0.38) — not backfilled, on purpose. A check that fails on the day
# it ships, for gaps everyone already knows about and has decided not to
# close yet, is a check people learn to ignore — and there is no way to
# notice a check that someone quietly stopped running.
#
# Not wired into CI — run manually, same shape as
# scripts/check-version-snippets.sh (RFC-074) and
# scripts/check-built-links.py (RFC-073).
#
# Usage: scripts/check-migration-guides.sh

cd "$(git rev-parse --show-toplevel)"

ADOPTION_MINOR=39

# Every distinct X.Y that has at least one git tag X.Y.Z, oldest first.
minors=$(git tag --list | grep -E '^[0-9]+\.[0-9]+\.[0-9]+$' \
  | awk -F. '{print $1"."$2}' | sort -t. -k1,1n -k2,2n -u)

prev=""
gaps_total=0
gaps_post_adoption=0

while IFS= read -r minor; do
  if [[ -n "$prev" ]]; then
    guide="docs/src/guides/migration-${prev}-to-${minor}.md"
    if [[ ! -f "$guide" ]]; then
      gaps_total=$((gaps_total + 1))
      curr_minor_num="${minor#*.}"
      if [[ "$curr_minor_num" -ge "$ADOPTION_MINOR" ]]; then
        echo "GAP (post-adoption, fails)   ${prev} -> ${minor}   missing: $guide"
        gaps_post_adoption=$((gaps_post_adoption + 1))
      else
        echo "gap (known historical)       ${prev} -> ${minor}   missing: $guide"
      fi
    fi
  fi
  prev="$minor"
done <<<"$minors"

echo
echo "$gaps_total total gap(s) found; $gaps_post_adoption at or after 0.$ADOPTION_MINOR."
if [[ $gaps_post_adoption -gt 0 ]]; then
  echo "FAIL: post-adoption gap(s) — every minor from 0.$ADOPTION_MINOR onward must ship a guide." >&2
  exit 1
fi
echo "OK — no gaps at or after 0.$ADOPTION_MINOR (the rule's adoption version)."

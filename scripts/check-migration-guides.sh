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
# Wired into CI's docs job (RFC-087), alongside
# scripts/check-version-snippets.sh (RFC-074) and
# scripts/check-built-links.py (RFC-073).
#
# REQUIRES TAGS. Those two read the working tree; this one derives its
# entire input from `git tag`. `actions/checkout` fetches no tags by
# default, so the docs job pins `fetch-depth: 0` — see the D-1 note
# below and .github/workflows/ci.yaml.
#
# Usage: scripts/check-migration-guides.sh

cd "$(git rev-parse --show-toplevel)"

ADOPTION_MINOR=39

# Every distinct X.Y that has at least one git tag X.Y.Z, oldest first.
#
# D-1 (2026-09-02): this pipeline used to run unguarded. With no tags
# visible, `grep` matches nothing and exits 1, `pipefail` makes the
# pipeline 1, and `set -e` killed the script before its first `echo` —
# a silent, instant, output-free exit 1. That is how it failed on every
# CI run from c651e93 to f153a2b without anyone reading it as a gate
# failure.
#
# `|| true` keeps the pipeline from aborting, so the emptiness check
# below can speak. **An empty tag list is a broken environment, not a
# clean bill of health**: with no tags this script sees zero minors,
# finds zero gaps, and would otherwise exit 0 — a green check that
# verified nothing, which is worse than the crash it replaced.
minors=$(git tag --list | grep -E '^[0-9]+\.[0-9]+\.[0-9]+$' \
  | awk -F. '{print $1"."$2}' | sort -t. -k1,1n -k2,2n -u || true)

if [[ -z "$minors" ]]; then
  echo "FAIL: no release tags visible — this check cannot run." >&2
  echo >&2
  echo "  Tags present in this checkout: $(git tag --list | wc -l)" >&2
  echo "  Expected: tags named X.Y.Z (bare, no 'v' prefix)." >&2
  echo >&2
  echo "In CI this means the checkout fetched no tags; the docs job needs" >&2
  echo "  - uses: actions/checkout@v6" >&2
  echo "    with:" >&2
  echo "      fetch-depth: 0" >&2
  echo >&2
  echo "Locally, run 'git fetch --tags' first." >&2
  exit 1
fi

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

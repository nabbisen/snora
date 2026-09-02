#!/usr/bin/env bash
set -euo pipefail

# RFC-092 Q-1: the gate half of check-docs-only.sh. Reads the given
# commit's own message for a `Docs-only: yes` trailer; if present, runs
# check-docs-only.sh against that commit and fails when it refutes the
# claim. If absent, this script has no opinion at all -- "No trailer,
# no check" is the rule Q-1 ruled for, precisely so this gate cannot
# become the kind of blanket policy Q-1's own RFC argued against
# ("not evidence for every sentence").
#
# Usage: scripts/check-docs-only-claim.sh <commit-sha>
#
# Exit codes:
#   0  no Docs-only claim on this commit, or the claim holds
#   1  the commit is unknown, or it claims Docs-only and does not hold

cd "$(git rev-parse --show-toplevel)"

SHA="${1:?usage: check-docs-only-claim.sh <commit-sha>}"

MESSAGE=$(git log -1 --format=%B "$SHA" 2>/dev/null || true)
if [[ -z "$MESSAGE" ]]; then
  echo "error: '$SHA' is not a commit this repository knows about" >&2
  exit 1
fi

# Trailer form: a line reading exactly "Docs-only: yes" (case-sensitive,
# matching git's own trailer convention of Key: value at the end of the
# message). Anything else -- "Docs-only: no", a typo, prose mentioning
# the words -- makes no claim this script will check.
if ! echo "$MESSAGE" | grep -qE '^Docs-only:[[:space:]]*yes[[:space:]]*$'; then
  echo "SKIP: $SHA makes no 'Docs-only: yes' claim"
  exit 0
fi

OUTPUT=$(scripts/check-docs-only.sh "$SHA")
if [[ -n "$OUTPUT" ]]; then
  echo "REFUSED: $SHA claims 'Docs-only: yes' but changes non-comment code under crates/:"
  echo "$OUTPUT"
  exit 1
fi

echo "OK: $SHA's 'Docs-only: yes' claim holds"

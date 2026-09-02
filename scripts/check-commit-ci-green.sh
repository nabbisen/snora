#!/usr/bin/env bash
set -euo pipefail

# RFC-090: refuses unless the given commit has an existing, completed,
# successful run of the CI workflow. Requires the *existing* run rather
# than re-running the suites itself (Q-5) -- re-running is slow and,
# worse, can pass on a commit whose earlier run failed for a reason the
# re-run doesn't reproduce (a flaky external dependency, a transient
# runner issue). Fails closed: no run found is a refusal, not a pass --
# the case a naive implementation treats as "nothing said no," which is
# exactly how the migration-guide gate (RFC-087 D-1) came to pass while
# checking nothing.
#
# Usage: scripts/check-commit-ci-green.sh <commit-sha> [workflow-file]
#
#   <commit-sha>    the commit to check (full or short SHA -- normalized
#                    to full before querying, see R-1 below)
#   [workflow-file] defaults to ci.yaml
#
# Requires: `gh` (authenticated, read access to this repo's Actions
# runs) and `jq`. JSON is parsed with jq rather than grep/cut -- a
# grep-based extraction that can legitimately match zero times (an
# empty run list) combines badly with `set -o pipefail`: grep's own
# exit 1 on no-match becomes the pipeline's exit status even when a
# later stage (e.g. `wc -l`) succeeds, killing the script under `set
# -e` before the "no run found" branch below ever runs -- the exact
# silent-death shape RFC-088's review round 1 found in this project's
# other new script the same week. jq's own -e flag would have the same
# problem; every `gh api` call below is followed by `|| echo '{...}'`
# instead, so a failed or empty query becomes a value this script's own
# logic can react to, not a `set -e` exit with nothing said.
#
# Exit codes:
#   0  a completed, successful run exists for this commit
#   1  no run exists, or the most recent run did not succeed

cd "$(git rev-parse --show-toplevel)"

SHA_ARG="${1:?usage: check-commit-ci-green.sh <commit-sha> [workflow-file]}"
WORKFLOW="${2:-ci.yaml}"

# R-1 (round 1 review, 2026-09-02): the GitHub API's `head_sha` filter
# matches the full 40-character SHA as an exact string, so a short SHA
# silently matched zero runs -- reading identically to "no run exists,"
# a different fact this script's own message claims to state. Normalize
# here so the documented "full or short SHA" behaviour is actually true
# rather than retracted.
#
# `--verify` specifically, not a bare `git rev-parse`: for an
# unrecognized argument, plain `git rev-parse` prints the *literal input
# back* on stdout (its best-effort path/revision fallback) while still
# exiting 128 -- caught while testing this exact fix, since checking
# `[[ -z "$SHA" ]]` after that would never fire (the "empty" branch
# below would silently never run; the garbage argument would instead
# sail through to the API call and get reported as an ordinary "no run
# found," burying a usage error inside a fail-closed message a second
# time in the same script). `--verify` returns truly empty stdout on
# failure, so checking emptiness here means what it says.
SHA=$(git rev-parse --verify "$SHA_ARG" 2>/dev/null || true)
if [[ -z "$SHA" ]]; then
  echo "error: '$SHA_ARG' is not a commit this repository knows about" >&2
  exit 1
fi

REPO=$(gh repo view --json nameWithOwner --jq .nameWithOwner)

# Ask for runs of this specific workflow, filtered to this commit, most
# recent first -- `head_sha` matches the commit the run was triggered
# for, regardless of which branch/PR carried it. Falls back to an empty
# list on any API error (network, rate limit, auth) rather than letting
# that surface as an unrelated shell failure.
RUNS_JSON=$(gh api "repos/$REPO/actions/workflows/$WORKFLOW/runs?head_sha=$SHA&per_page=1" 2>/dev/null || echo '{"workflow_runs":[]}')

RUN_COUNT=$(echo "$RUNS_JSON" | jq '.workflow_runs | length')

if [[ "$RUN_COUNT" -eq 0 ]]; then
  echo "REFUSED: no run of $WORKFLOW found for commit $SHA -- fail closed, not a pass." >&2
  exit 1
fi

STATUS=$(echo "$RUNS_JSON" | jq -r '.workflow_runs[0].status')
CONCLUSION=$(echo "$RUNS_JSON" | jq -r '.workflow_runs[0].conclusion')
URL=$(echo "$RUNS_JSON" | jq -r '.workflow_runs[0].html_url')

if [[ "$STATUS" != "completed" ]]; then
  echo "REFUSED: most recent $WORKFLOW run for commit $SHA is '$STATUS', not completed -- $URL" >&2
  exit 1
fi

if [[ "$CONCLUSION" != "success" ]]; then
  echo "REFUSED: most recent $WORKFLOW run for commit $SHA concluded '$CONCLUSION', not success -- $URL" >&2
  exit 1
fi

echo "OK: $WORKFLOW succeeded for commit $SHA -- $URL"

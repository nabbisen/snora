#!/usr/bin/env bash
set -euo pipefail

# Audit 2026-09-12 (C-1 / 0.47.0 Unit 4). AA_TEXT, NON_TEXT_MIN,
# FOCUS_MIN, and TEXT_SIZE_MIN are deliberately duplicated once per crate
# that needs them -- the engine (`snora`) depends on neither
# `snora-design` nor `snora-widgets` by design, so these genuinely cannot
# be shared through a common module. The duplication is correct and must
# stay; the drift it makes possible is not.
#
# `NON_TEXT_MIN` has already drifted once: to `1.3` instead of `3.0`,
# unnoticed until an RFC-071 review caught it. The guard adopted at the
# time was a comment (`crates/snora/src/toast/contrast_tests.rs`) naming
# the constant's siblings so a future editor would know to check them --
# and that comment is now itself wrong about its own scope: it names two
# siblings, and there are three (a fourth copy, in
# `crates/snora/src/design/render/tests.rs`, exists and was not added to
# it). A comment that has to be kept in sync by hand is exactly the
# mechanism this project has spent RFC-090 through RFC-096 replacing.
#
# This script is that replacement: extract every copy of each named
# constant under `crates/`, fail if a name resolves to more than one
# distinct value, and fail if the number of copies for a name moves away
# from what is pinned below. An added or removed copy is a fact nobody
# decided until this script says so -- the same reasoning RFC-063's
# exhaustive enum matches apply to a type the compiler can check, applied
# here to a textual convention the compiler cannot.
#
# Usage: scripts/check-wcag-floors.sh

cd "$(git rev-parse --show-toplevel)"

# name -> expected copy count under crates/, pinned deliberately. A
# pinned count costs an edit here whenever a copy is legitimately added
# or removed -- accepted on purpose: the alternative is a script that
# only ever notices a *value* mismatch, which would have caught the
# 1.3-drift incident but not the uncounted-fourth-copy one, and this
# audit found both are real failure modes for the same set of constants.
declare -A EXPECTED_COUNT=(
  [AA_TEXT]=3
  [NON_TEXT_MIN]=4
  [FOCUS_MIN]=1
  [TEXT_SIZE_MIN]=1
)

fail=0

for name in AA_TEXT NON_TEXT_MIN FOCUS_MIN TEXT_SIZE_MIN; do
  # `|| true` is required here: under `set -e`, a zero-match grep's own
  # exit status would otherwise kill the script before the `-z` check
  # below ever runs -- the exact silent-death shape this project has hit
  # in `check-workspace-iced-features.sh` and `check-commit-ci-green.sh`.
  matches=$(grep -rnE "const ${name}: f32 = [0-9.]+;" crates/ || true)

  if [[ -z "$matches" ]]; then
    echo "FAIL: $name -- no copies found under crates/ (expected ${EXPECTED_COUNT[$name]})"
    fail=1
    continue
  fi

  count=$(echo "$matches" | wc -l)
  values=$(echo "$matches" | sed -E "s/.*const ${name}: f32 = ([0-9.]+);.*/\1/" | sort -u)
  distinct=$(echo "$values" | wc -l)

  if [[ "$distinct" -gt 1 ]]; then
    echo "FAIL: $name has $distinct distinct values across its copies:"
    echo "$matches" | sed 's/^/  /'
    fail=1
  elif [[ "$count" -ne "${EXPECTED_COUNT[$name]}" ]]; then
    echo "FAIL: $name has $count cop$([[ $count -eq 1 ]] && echo y || echo ies) under crates/, expected ${EXPECTED_COUNT[$name]} -- a copy was added or removed without updating this script's pinned count:"
    echo "$matches" | sed 's/^/  /'
    fail=1
  else
    echo "OK: $name -- $count cop$([[ $count -eq 1 ]] && echo y || echo ies), all agree at $values"
  fi
done

if [[ "$fail" -ne 0 ]]; then
  exit 1
fi

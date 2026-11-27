#!/usr/bin/env bash
set -euo pipefail

# 0.65 (RFC-134 Unit 1 review, Q-1 and its addendum). The rendered tiny-skia tests of `snora` run in TWO jobs with all
# features, so that the three sweeps (the longest part of a job) do not sit on the critical path with everything else, and
# in one job without `lucide-icons`:
#
#   all features       rendered-tiny-skia                    the library's unit tests and every target but the sweeps
#                      rendered-sweeps-tiny-skia             the sweeps
#   no lucide-icons    rendered-tiny-skia-no-lucide          the same as the first, and the doc-tests, as the feature set
#
# **The sweeps are not run without `lucide-icons`** (RFC-139 round 3). Since RFC-139 the marks snora draws are the same text
# glyphs in every build, the non-text table the sweeps record is identical row for row with and without the feature, and
# `lucide_marks_in_the_starting_state.rs` (in `rest`, so it runs in both) guards that they cannot diverge again. Before,
# the two combinations drew different marks and each had its sweeps.
#
# **Each integration-test target runs in exactly one job of the all-features set, and every one but the sweeps in the
# no-lucide job, by construction:** this script is the one place the sweeps are named, and it prints cargo's flags for each
# half. The wgpu job runs everything (`--tests`), as it did.
#
#   scripts/rendered-targets.sh rest all-features|no-lucide, or sweeps all-features
#                  --test a --test b ...   (`rest`: every target that is not a sweep; add `--lib` and `--doc` yourself)
#   scripts/rendered-targets.sh check
#                  exits non-zero unless the two halves partition `crates/snora/tests/*.rs`, every sweep named here exists,
#                  and ci.yaml uses each of the three (half, feature set) once
#
# The feature set is a name and not a flag: the targets are the same files under either, and what the argument does is
# make ci.yaml say which combination each job is, so that `check` can count them.
#
# **The direction of a mistake is the safe one.** A new sweep that is not named here runs in `rest` (slower, never
# skipped); a name that is not a test file fails `check`. A developer's `cargo test -p snora --all-features` still runs
# everything: nothing is marked `#[ignore]`.

cd "$(git rev-parse --show-toplevel)"

# The sweeps: read the drawn pixels of every scene in every preset, on every surface, in every state. Measured at 0.65
# on tiny-skia: nontext_sweep 281 s, text_sweep 83 s, form_unstyled_drawn 53 s.
SWEEPS=(nontext_sweep text_sweep form_unstyled_drawn)

targets() { # every integration-test target of snora: tests/*.rs, which cargo discovers by itself
  local f
  for f in crates/snora/tests/*.rs; do basename "$f" .rs; done
}

feature_set_ok() { [ "$1" = all-features ] || [ "$1" = no-lucide ]; }

is_sweep() {
  local s
  for s in "${SWEEPS[@]}"; do [ "$s" = "$1" ] && return 0; done
  return 1
}

flags() { # rest | sweeps
  local t out=()
  feature_set_ok "${2:-}" || { echo "usage: scripts/rendered-targets.sh rest all-features|no-lucide, or sweeps all-features" >&2; exit 2; }
  # The sweeps are not run without `lucide-icons` (RFC-139 round 3): there is no such half to print.
  [ "$1" = sweeps ] && [ "$2" = no-lucide ] && { echo "rendered-targets.sh: the sweeps do not run in the no-lucide combination (RFC-139 round 3)" >&2; exit 2; }
  for t in $(targets); do
    if is_sweep "$t"; then
      [ "$1" = sweeps ] && out+=(--test "$t")
    else
      [ "$1" = rest ] && out+=(--test "$t")
    fi
  done
  echo "${out[@]}"
}

check() {
  local ok=0 s t n=0 rest sweeps
  for s in "${SWEEPS[@]}"; do
    if [ ! -f "crates/snora/tests/$s.rs" ]; then
      echo "rendered-targets.sh: the sweep \`$s\` is not a test file (crates/snora/tests/$s.rs): remove it from SWEEPS" >&2
      ok=1
    fi
  done
  rest=$(flags rest all-features | tr ' ' '\n' | grep -v '^--test$' || true)
  sweeps=$(flags sweeps all-features | tr ' ' '\n' | grep -v '^--test$' || true)
  for t in $(targets); do
    n=$((n + 1))
    in_rest=$(printf '%s\n' "$rest" | grep -cx "$t" || true)
    in_sweeps=$(printf '%s\n' "$sweeps" | grep -cx "$t" || true)
    if [ $((in_rest + in_sweeps)) -ne 1 ]; then
      echo "rendered-targets.sh: the target \`$t\` is in $((in_rest + in_sweeps)) lists, not one" >&2
      ok=1
    fi
  done
  [ "$n" -gt 30 ] || { echo "rendered-targets.sh: only $n test targets found: the glob stopped reading" >&2; ok=1; }
  # ci.yaml uses each of the three (half, feature set) once.
  local ci=.github/workflows/ci.yaml pair half set
  for pair in "rest all-features" "rest no-lucide" "sweeps all-features"; do
    half=${pair% *}; set=${pair#* }
    [ "$(grep -c "scripts/rendered-targets.sh $half $set" "$ci")" -eq 1 ] \
      || { echo "rendered-targets.sh: ci.yaml must run \`scripts/rendered-targets.sh $half $set\` exactly once" >&2; ok=1; }
  done
  [ "$(grep -c "scripts/rendered-targets.sh sweeps no-lucide" "$ci")" -eq 0 ] \
    || { echo "rendered-targets.sh: ci.yaml runs the sweeps without lucide-icons, which RFC-139 round 3 dropped" >&2; ok=1; }
  # The no-lucide combination also runs the doc-tests (a plain `cargo test` did; `--lib --test ..` do not).
  [ "$(grep -c -- '--features widgets,design,iced-default --doc' "$ci")" -eq 1 ] \
    || { echo "rendered-targets.sh: ci.yaml must run the doc-tests of the no-lucide combination exactly once" >&2; ok=1; }
  [ "$ok" -eq 0 ] && echo "rendered-targets.sh: $n test targets, ${#SWEEPS[@]} of them sweeps; with all features each is in exactly one job, without lucide-icons every one but the sweeps is in the one job"
  return "$ok"
}

case "${1:-}" in
  rest | sweeps) flags "$1" "${2:-}" ;;
  check) check ;;
  *) echo "usage: scripts/rendered-targets.sh rest all-features|no-lucide, sweeps all-features, or check" >&2; exit 2 ;;
esac

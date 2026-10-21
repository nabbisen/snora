#!/usr/bin/env bash
set -euo pipefail

# RFC-088 (Q-3): every feature the workspace's own [workspace.dependencies]
# `iced` line turns on must be used by something in this workspace, or it
# forces build cost on every consumer for a capability nobody asked for.
# This is the general property RFC-083's gate does not check — that gate
# asserts a crate has no iced dependency at all; this one asserts the
# features iced IS asked for are not dead weight. Exactly the shape that let
# `canvas` (zero occurrences) and `svg` (used only under our own svg-icons
# feature, which already declares it independently) both survive RFC-083 by
# one line.
#
# Wired into CI (RFC-087's own rule: a gate nothing runs is not a gate).
#
# Usage: scripts/check-workspace-iced-features.sh

cd "$(git rev-parse --show-toplevel)"

# One feature is exempt from the usage grep below: `tokio` supplies iced's
# async executor. `crates/snora/src/toast.rs`'s `subscription()` (public,
# unconditional -- no snora feature gates it) calls `iced::time::every`,
# which does not exist at all without an executor feature compiled into
# iced -- confirmed empirically by removing "tokio" from this line and
# reproducing `error[E0425]: cannot find function 'every' in module
# 'iced::time'`. This is a structural requirement, not a textual usage a
# grep can see (nothing in this workspace's source literally says
# "tokio"), so it cannot be verified the same way canvas/svg can and is
# named here explicitly rather than silently exempted.
#
# `web-colors` and `crisp` are exempt for a different reason, and
# RFC-112 states it as a rule rather than a case: **a feature that
# decides how snora draws is snora's; a feature that decides what
# draws, or where it runs, is the application's.** These two are the
# first kind — snora's colour space and its pixel alignment, which
# every measured contrast figure in the test suite assumes — so they
# are named on the workspace line and stay on for every consumer.
# Neither appears as `iced::widget::<name>` at a call site, because
# neither is a widget; they change how existing widgets are drawn.
# Everything of the second kind reaches iced through each crate's
# `iced-default` feature instead, which an application can turn off.
EXEMPT_FEATURES=("tokio" "web-colors" "crisp")

# Matches on `^iced = {` alone, not a pinned iced version. R-1 (round 1
# review, 2026-09-02): the pattern used to require `version = "0.14"`
# literally, so an iced upgrade -- the exact event this gate is meant to
# survive -- would make it match nothing. Combined with `set -euo
# pipefail`, that made the grep's own failure kill the script before the
# `if` below could ever run, so the "could not find the line" message
# was dead code: the gate went red with no output at all. The `|| true`
# on the substitution is what lets that message actually fire now --
# without it, `set -e` claims the failure first every time.
# The entry may be written on one line or spread over several (RFC-112
# put the feature list on its own lines), so take everything from
# `iced = {` to the line that closes the table and fold it into one.
#
# **The single-line version of this read as a pass on a multi-line
# entry**, which is the failure this project keeps meeting: the sed
# below found no `features = [...]` on the first line, `FEATURES_RAW`
# came out empty, and the script printed "declares no extra features"
# and exited 0 with three features declared right under it. That is
# RFC-087's D-1 again — a gate that passed because it checked nothing.
ICED_BLOCK=$(awk '
  /^iced = \{/ { inside = 1 }
  inside        { block = block $0 " " }
  inside && /\}[[:space:]]*$/ { print block; exit }
' Cargo.toml)

if [[ -z "$ICED_BLOCK" ]]; then
  echo "error: could not find the workspace [workspace.dependencies] iced entry in Cargo.toml" >&2
  exit 1
fi

FEATURES_RAW=$(echo "$ICED_BLOCK" | sed -nE 's/.*features = \[([^]]*)\].*/\1/p')

if [[ -z "$FEATURES_RAW" ]]; then
  # Distinguish "no features key" from "a features key this could not
  # read". The first is a legitimate pass; the second is the vacuous
  # one above, and must be loud.
  if grep -q 'features' <<<"$ICED_BLOCK"; then
    echo "error: the iced entry declares features, but this script could not parse them:" >&2
    echo "  $ICED_BLOCK" >&2
    exit 1
  fi
  echo "OK: workspace iced entry declares no extra features."
  exit 0
fi

# Split "\"a\", \"b\"" into a bash array: a b. Quotes are stripped before
# splitting on comma, and whitespace is trimmed per element after -- doing
# it in the other order would delete the newlines `tr ',' '\n'` just
# introduced (`[:space:]` matches newline too), silently re-merging every
# element into one.
readarray -t FEATURES < <(echo "$FEATURES_RAW" | tr -d '"' | tr ',' '\n' | sed 's/^[[:space:]]*//;s/[[:space:]]*$//')

unused=0
checked=0

is_exempt() {
  local feature="$1"
  for exempt in "${EXEMPT_FEATURES[@]}"; do
    if [[ "$feature" == "$exempt" ]]; then
      return 0
    fi
  done
  return 1
}

for feature in "${FEATURES[@]}"; do
  [[ -z "$feature" ]] && continue
  checked=$((checked + 1))

  if is_exempt "$feature"; then
    echo "exempt (structural, see script comment): $feature"
    continue
  fi

  # An iced feature that gates a widget or capability shows up as
  # `iced::widget::<feature>` (or the shorthand `widget::<feature>` after a
  # `use iced::widget` import) at its call site -- the same pattern
  # confirmed for `svg` (icon.rs:45) before this feature was removed from
  # this line. This will not find a feature used only via some other
  # naming shape; if that ever happens, extend the pattern rather than
  # widen EXEMPT_FEATURES to hide it.
  if grep -rq --include="*.rs" -- "iced::widget::${feature}\|widget::${feature}\b" crates/ examples/; then
    echo "used: $feature"
  else
    echo "UNUSED: $feature -- declared on the workspace iced line, not found in use"
    unused=$((unused + 1))
  fi
done

echo
echo "Checked $checked feature(s), $unused unused."

if [[ "$unused" -gt 0 ]]; then
  exit 1
fi

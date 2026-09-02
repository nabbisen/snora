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
EXEMPT_FEATURES=("tokio")

# Matches on `^iced = {` alone, not a pinned iced version. R-1 (round 1
# review, 2026-09-02): the pattern used to require `version = "0.14"`
# literally, so an iced upgrade -- the exact event this gate is meant to
# survive -- would make it match nothing. Combined with `set -euo
# pipefail`, that made the grep's own failure kill the script before the
# `if` below could ever run, so the "could not find the line" message
# was dead code: the gate went red with no output at all. The `|| true`
# on the substitution is what lets that message actually fire now --
# without it, `set -e` claims the failure first every time.
ICED_LINE=$(grep -m1 '^iced = {' Cargo.toml || true)
if [[ -z "$ICED_LINE" ]]; then
  echo "error: could not find the workspace [workspace.dependencies] iced line in Cargo.toml" >&2
  exit 1
fi

FEATURES_RAW=$(echo "$ICED_LINE" | sed -nE 's/.*features = \[([^]]*)\].*/\1/p')

if [[ -z "$FEATURES_RAW" ]]; then
  echo "OK: workspace iced line declares no extra features."
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

#!/usr/bin/env bash
set -euo pipefail

# RFC-104 R-4. Every chrome and toast label used to carry a literal text
# size -- 13 in the tab bar and breadcrumb, 14 in menus, 16 in the header
# title, 16/14 in toasts. An application that set iced's
# `default_text_size`, or that shipped its own `Typography`, got those
# numbers anyway. RFC-104 removed them; this script is what stops them
# coming back one call at a time.
#
# Two rules, because one is not enough:
#
#   1. `.size(` with a numeric argument is a literal, wherever it appears
#      in non-test source.
#   2. `.size(` with an UPPER_CASE constant argument is a literal behind a
#      name unless that name is allow-listed below. Without this rule
#      `const LABEL: f32 = 13.0;` bypasses rule 1 and reads as tidier code
#      than the literal it hides (Q-4).
#
# A token helper call -- `snora_style::text::label_size(tokens)` -- and a
# plain variable are allowed structurally: neither matches either rule.
# That is the intended shape, so the script says nothing about them.
#
# Usage: scripts/check-literal-sizes.sh

cd "$(git rev-parse --show-toplevel)"

# Allow-listed constant -> the one-line reason it is not a text size.
# Named constants only: a bare number is never allow-listed, because the
# point of the allow-list is that someone had to write down why.
declare -A ALLOWED=(
  [CLOSE_GLYPH_SIZE]="the toast close control's glyph metric (RFC-101), sized to the control's 24x24 target rather than to running text"
)

# Non-test source only. The exclusions are by filename because the test
# modules in these crates are whole files:
#   tests.rs / *_tests.rs -- unit-test modules
#   *_register.rs         -- the channel and state registers, both `#[cfg(test)]`
mapfile -t FILES < <(
  find crates/snora-widgets/src crates/snora/src -name '*.rs' \
    ! -name 'tests.rs' ! -name '*_tests.rs' ! -name '*_register.rs' | sort
)

failures=()

# Each `.size(` call's argument, read to the end of its line. The
# argument is everything up to the call's closing paren: strip that paren
# and whatever follows it.
#
# Three ways to fail, in order:
#
#   1. the whole argument is an allow-listed constant -- allowed;
#   2. the argument contains a digit anywhere -- a literal, wherever it
#      sits. `.size(iced::Pixels(13.0))` hid one from an earlier version
#      of this script that only looked at the argument's first character;
#   3. the argument is an UPPER_CASE constant that is not allow-listed --
#      a literal behind a name.
#
# A digit anywhere is deliberately blunt: no legitimate call in this
# codebase has one, because a size is either a variable or a token helper
# (`snora_style::text::label_size(tokens)`). If a future call genuinely
# needs one -- a tuple access like `foo.0`, say -- give it a named
# constant and allow-list it with the reason, which is the outcome this
# script is for.
for file in "${FILES[@]}"; do
  # `grep` exits 1 when a file has no match, which is the common case and
  # not an error -- `|| true` keeps `set -e` from killing the run, and the
  # emptiness of the result is what is checked instead.
  hits=$(grep -n '\.size(' "$file" || true)
  [[ -z "$hits" ]] && continue

  while IFS= read -r hit; do
    lineno=${hit%%:*}
    code=${hit#*:}

    # Text after the first `.size(`, with the closing paren and anything
    # after it removed.
    arg=${code#*.size(}
    arg=$(printf '%s' "$arg" | sed -E 's/[[:space:]]*\)[^)]*$//' | sed -E 's/^[[:space:]]+|[[:space:]]+$//g')

    if [[ -n "${ALLOWED[$arg]+set}" ]]; then
      continue
    fi

    if [[ -z "$arg" ]]; then
      failures+=("$file:$lineno -- .size( with its argument on another line; keep it on one line so this check can read it")
    elif [[ "$arg" =~ [0-9] ]]; then
      failures+=("$file:$lineno -- .size($arg): a numeric literal")
    elif [[ "$arg" =~ ^[A-Z][A-Z0-9_]*$ ]]; then
      failures+=("$file:$lineno -- .size($arg): an upper-case constant that is not allow-listed")
    fi
  done <<<"$hits"
done

if ((${#failures[@]} > 0)); then
  echo "literal text sizes found (${#failures[@]}):" >&2
  printf '  %s\n' "${failures[@]}" >&2
  cat >&2 <<'EOF'

A label's size belongs to its host: `default_text_size` on the unstyled
path, a `Typography` role on the styled one. Carry it in through the
widget's geometry, as spacing already is, rather than writing a number
here. If the value genuinely is not a text size -- a glyph metric, say --
add its constant to this script's allow-list with the reason.
EOF
  exit 1
fi

echo "literal sizes ok: ${#FILES[@]} non-test source files scanned, ${#ALLOWED[@]} allow-listed constant(s)"

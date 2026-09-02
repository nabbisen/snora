#!/usr/bin/env bash
set -euo pipefail

# RFC-092 Part 1A: "documentation only" / "no behaviour change" is a
# claim, not a fact -- this makes it a command. Prints every changed
# line under `crates/` in the given revision that is not blank and not
# a line comment. Empty output is what a docs-only claim looks like;
# any output refutes it.
#
# This is the exact filter that found F-33 (a dead match in tab.rs) and
# F-34 (sheet.rs's portions() extraction) by hand, after 0.41.1 had
# already shipped claiming "no behaviour change" (RFC-089's own commit,
# 2f83e72).
#
# Usage: scripts/check-docs-only.sh <rev>
#
#   <rev>  a commit-ish. Checked against its own parent (<rev>^..<rev>)
#          -- the change that commit itself introduced, not everything
#          since some other point.
#
# What "comment" means here, and what it does not:
#
#   Recognizes `//`, `///`, and `//!` line comments (this codebase's
#   only comment style -- confirmed, no `/* */` block comment appears
#   anywhere under crates/ as of this writing). A block comment would
#   not be recognized line-by-line by this heuristic and would show up
#   as false-positive "code" -- the safe direction to be wrong in for a
#   tool whose job is to refute a claim, not confirm one. Stated here
#   rather than silently assumed away.
#
#   Does NOT distinguish a doc comment's own prose from a code example
#   inside it (a ```rust fenced block in a `///` comment is still every
#   line prefixed `///`, so it reads as a comment line here too, even
#   though rustdoc compiles and runs it as a real test). RFC-089's own
#   F-23 added exactly such a doctest and this script does not flag it
#   -- an accepted simplification of Part 1A's own literal definition
#   ("non-comment, non-blank"), not an oversight; a doctest's own
#   compile/run behavior is covered by `cargo test`, not by this claim
#   check.
#
# Exit codes:
#   0  always -- this is a reporting tool. The gate built on top of it
#      (a CI job reading a "Docs-only: yes" trailer) is what turns
#      non-empty output into a failure; see scripts/README.md.

cd "$(git rev-parse --show-toplevel)"

REV="${1:?usage: check-docs-only.sh <rev>}"

git diff --no-color -U0 "$REV^" "$REV" -- crates/ | awk '
  /^\+\+\+ / {
    file = $0
    sub(/^\+\+\+ b\//, "", file)
    next
  }
  /^--- / { next }
  /^diff --git/ { next }
  /^index / { next }
  /^@@/ { next }
  /^\+/ || /^-/ {
    marker = substr($0, 1, 1)
    content = substr($0, 2)
    trimmed = content
    sub(/^[ \t]+/, "", trimmed)
    if (trimmed == "") next
    if (trimmed ~ /^\/\//) next
    print file ":" marker content
  }
'

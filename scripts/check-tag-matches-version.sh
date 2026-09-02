#!/usr/bin/env bash
set -euo pipefail

# RFC-090: refuses unless the given tag matches [workspace.package].version
# in Cargo.toml exactly. Prints both values either way, so a mismatch
# names what disagreed rather than just failing.
#
# Usage: scripts/check-tag-matches-version.sh <tag>
#
# Exit codes:
#   0  tag matches Cargo.toml's version
#   1  tag does not match, or Cargo.toml's version could not be read

cd "$(git rev-parse --show-toplevel)"

TAG="${1:?usage: check-tag-matches-version.sh <tag>}"

CARGO_VERSION=$(grep -m1 '^version = "[0-9]\+\.[0-9]\+\.[0-9]\+"' Cargo.toml \
  | sed -E 's/.*"([0-9]+\.[0-9]+\.[0-9]+)".*/\1/' || true)

if [[ -z "$CARGO_VERSION" ]]; then
  echo "REFUSED: could not read [workspace.package].version from Cargo.toml" >&2
  exit 1
fi

if [[ "$TAG" != "$CARGO_VERSION" ]]; then
  echo "REFUSED: tag '$TAG' does not match [workspace.package].version '$CARGO_VERSION' in Cargo.toml" >&2
  exit 1
fi

echo "OK: tag '$TAG' matches [workspace.package].version '$CARGO_VERSION'"

#!/usr/bin/env bash
#
# measure-binary-size.sh — produce a single CSV row describing the
# release-build binary size of `examples/hello`, both with and
# without the optional `widgets` feature.
#
# Usage:
#
#     scripts/measure-binary-size.sh <version> [<lto-mode>]
#
# Arguments:
#
#   <version>   — the snora version this measurement refers to,
#                 written verbatim into the CSV.  E.g. "0.10.0".
#
#   <lto-mode>  — optional, defaults to "off".  Written verbatim
#                 into the CSV's lto column.  This script always
#                 builds with the workspace's release-baseline
#                 profile, which has lto = false; the argument
#                 exists so callers can record a different value
#                 if they switch profiles.
#
# Output (stdout):
#
#   One CSV row matching the schema in
#   docs/src/reference/binary-size-budget/binary-size.csv :
#
#     version,widgets_on_bytes,widgets_off_bytes,diff_bytes,lto,date
#
# Exit code:
#
#   0 on success.
#   non-zero on any build / strip / measurement failure.
#
# This script is invoked by the `binary-size` GitHub Actions
# workflow on every push (job summary + artifact only) and on every
# release tag (output appended to the budget CSV via
# scripts/append-binary-size-row.sh).

set -euo pipefail

VERSION="${1:?missing version argument}"
LTO_MODE="${2:-off}"

# Always run from the workspace root, even when invoked from
# elsewhere.
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)"
WORKSPACE_DIR="$(cd -- "$SCRIPT_DIR/.." &>/dev/null && pwd)"
cd "$WORKSPACE_DIR"

PROFILE="release-baseline"
BIN_NAME="snora-example-hello"
BIN_PATH="target/$PROFILE/$BIN_NAME"

build_and_measure() {
    local label="$1"
    shift  # remaining args go to cargo

    # Clean only the binary's own artifacts so iced's heavy deps
    # stay cached between the two builds.  Without this the second
    # build re-uses the first build's binary and reports a stale
    # size.
    cargo clean -p "$BIN_NAME" --profile "$PROFILE" >&2

    cargo build --profile "$PROFILE" -p "$BIN_NAME" "$@" >&2

    if [[ ! -f "$BIN_PATH" ]]; then
        echo "build did not produce $BIN_PATH (label=$label)" >&2
        exit 1
    fi

    # Strip a copy so the original binary is left intact for any
    # subsequent step that wants it.
    local stripped="$BIN_PATH.stripped.$label"
    cp "$BIN_PATH" "$stripped"
    strip --strip-all "$stripped"

    stat -c '%s' "$stripped"
}

echo "Measuring widgets ON ..." >&2
WIDGETS_ON=$(build_and_measure "on")

echo "Measuring widgets OFF ..." >&2
WIDGETS_OFF=$(build_and_measure "off" --no-default-features)

DIFF=$(( WIDGETS_ON - WIDGETS_OFF ))
DATE=$(date -u +%Y-%m-%d)

# CSV row, matching the budget CSV header.
echo "${VERSION},${WIDGETS_ON},${WIDGETS_OFF},${DIFF},${LTO_MODE},${DATE}"

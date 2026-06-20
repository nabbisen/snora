#!/usr/bin/env bash
#
# measure-binary-size.sh — produce a single CSV row describing the
# release-build binary size of representative Snora binaries.
#
# Usage:
#
#     scripts/measure-binary-size.sh <version>
#
# Arguments:
#
#   <version>   — the snora version this measurement refers to,
#                 written verbatim into the CSV.  E.g. "0.24.0".
#
# Output (stdout):
#
#   One CSV row matching the schema in
#   docs/src/reference/binary-size-budget/binary-size.csv :
#
#     version,widgets_on_bytes,widgets_off_bytes,diff_bytes,design_on_bytes,design_diff_bytes,rustc,runner_os,date
#
#   Columns:
#     widgets_on_bytes   — snora-example-hello with default features (widgets ON)
#     widgets_off_bytes  — snora-example-hello with --no-default-features
#     diff_bytes         — widgets_on - widgets_off (marginal cost of widgets)
#     design_on_bytes    — snora-example-design-workbench (widgets + design)
#     design_diff_bytes  — design_on - widgets_on (marginal cost of design)
#     rustc              — toolchain version
#     runner_os          — CI runner OS
#     date               — UTC date of measurement
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

# Always run from the workspace root, even when invoked from
# elsewhere.
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)"
WORKSPACE_DIR="$(cd -- "$SCRIPT_DIR/.." &>/dev/null && pwd)"
cd "$WORKSPACE_DIR"

PROFILE="release-baseline"
HELLO_BIN="snora-example-hello"
WORKBENCH_BIN="snora-example-design-workbench"
HELLO_PATH="target/$PROFILE/$HELLO_BIN"
WORKBENCH_PATH="target/$PROFILE/$WORKBENCH_BIN"

build_and_measure() {
    local label="$1"
    local bin_name="$2"
    local bin_path="$3"
    shift 3  # remaining args go to cargo

    # Clean only the binary's own artifacts so iced's heavy deps
    # stay cached between builds.
    cargo clean -p "$bin_name" --profile "$PROFILE" >&2

    cargo build --profile "$PROFILE" -p "$bin_name" "$@" >&2

    if [[ ! -f "$bin_path" ]]; then
        echo "build did not produce $bin_path (label=$label)" >&2
        exit 1
    fi

    # Strip a copy so the original binary is left intact for any
    # subsequent step that wants it.
    local stripped="$bin_path.stripped.$label"
    cp "$bin_path" "$stripped"
    strip --strip-all "$stripped"

    stat -c '%s' "$stripped"
}

echo "Measuring widgets ON (snora-example-hello, default features) ..." >&2
WIDGETS_ON=$(build_and_measure "on" "$HELLO_BIN" "$HELLO_PATH")

echo "Measuring widgets OFF (snora-example-hello, --no-default-features) ..." >&2
WIDGETS_OFF=$(build_and_measure "off" "$HELLO_BIN" "$HELLO_PATH" --no-default-features)

echo "Measuring design ON (snora-example-design-workbench, widgets+design) ..." >&2
DESIGN_ON=$(build_and_measure "design" "$WORKBENCH_BIN" "$WORKBENCH_PATH")

DIFF=$(( WIDGETS_ON - WIDGETS_OFF ))
DESIGN_DIFF=$(( DESIGN_ON - WIDGETS_ON ))
DATE=$(date -u +%Y-%m-%d)
RUSTC=$(rustc --version | tr ' ' '_')
RUNNER_OS="${RUNNER_OS:-unknown}"

# CSV row, matching the budget CSV header.
echo "${VERSION},${WIDGETS_ON},${WIDGETS_OFF},${DIFF},${DESIGN_ON},${DESIGN_DIFF},${RUSTC},${RUNNER_OS},${DATE}"

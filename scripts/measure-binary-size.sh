#!/usr/bin/env bash
#
# measure-binary-size.sh — produce a single CSV row describing the
# release-build binary sizes of three size-probe binaries.
#
# Usage:
#
#     scripts/measure-binary-size.sh <version>
#
# Arguments:
#
#   <version>   — the snora version this measurement refers to,
#                 written verbatim into the CSV.  E.g. "0.25.0".
#
# Output (stdout):
#
#   One CSV row matching the schema in
#   docs/src/reference/binary-size-budget/binary-size.csv :
#
#     version,engine_bytes,widgets_bytes,widgets_diff_bytes,design_bytes,design_diff_bytes,rustc,runner_os,date
#
#   Columns:
#     engine_bytes        — snora-size-probe-engine (engine only, no widgets)
#     widgets_bytes       — snora-size-probe-widgets (default features)
#     widgets_diff_bytes  — widgets_bytes - engine_bytes (marginal cost of widgets)
#     design_bytes        — snora-size-probe-design (widgets + design)
#     design_diff_bytes   — design_bytes - widgets_bytes (marginal cost of design)
#     rustc               — Rust toolchain version string
#     runner_os           — CI runner OS
#     date                — UTC date of measurement (YYYY-MM-DD)
#
#   All three probes use identical application code (see examples/size_probe_*/src/main.rs),
#   so the diff values are purely the marginal cost of each feature, not of
#   different application logic.
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

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)"
WORKSPACE_DIR="$(cd -- "$SCRIPT_DIR/.." &>/dev/null && pwd)"
cd "$WORKSPACE_DIR"

PROFILE="release-baseline"

build_and_measure() {
    local label="$1"
    local bin_name="$2"
    local bin_path="target/$PROFILE/$bin_name"
    shift 2

    cargo clean -p "$bin_name" --profile "$PROFILE" >&2
    cargo build --profile "$PROFILE" -p "$bin_name" "$@" >&2

    if [[ ! -f "$bin_path" ]]; then
        echo "build did not produce $bin_path (label=$label)" >&2
        exit 1
    fi

    local stripped="$bin_path.stripped.$label"
    cp "$bin_path" "$stripped"
    strip --strip-all "$stripped"
    stat -c '%s' "$stripped"
}

echo "Measuring engine-only (snora-size-probe-engine) ..." >&2
ENGINE=$(build_and_measure "engine" "snora-size-probe-engine")

echo "Measuring widgets (snora-size-probe-widgets) ..." >&2
WIDGETS=$(build_and_measure "widgets" "snora-size-probe-widgets")

echo "Measuring design (snora-size-probe-design) ..." >&2
DESIGN=$(build_and_measure "design" "snora-size-probe-design")

WIDGETS_DIFF=$(( WIDGETS - ENGINE ))
DESIGN_DIFF=$(( DESIGN - WIDGETS ))
DATE=$(date -u +%Y-%m-%d)
RUSTC=$(rustc --version | tr ' ' '_')
RUNNER_OS="${RUNNER_OS:-unknown}"

echo "${VERSION},${ENGINE},${WIDGETS},${WIDGETS_DIFF},${DESIGN},${DESIGN_DIFF},${RUSTC},${RUNNER_OS},${DATE}"

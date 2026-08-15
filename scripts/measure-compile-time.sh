#!/usr/bin/env bash
# measure-compile-time.sh <version>
#
# Measures cold compile time for representative Snora build configurations
# and emits one CSV row to stdout.
#
# Usage:
#   scripts/measure-compile-time.sh 0.12.0
#
# Output (one line):
#   version,check_workspace_ms,build_widgets_ms,build_engine_only_ms,example_hello_ms,build_widgets_design_ms,example_workbench_ms,rustc,runner_os,date
#
#   runner_os is read from $SNORA_RUNNER_OS if set, else $RUNNER_OS, else
#   "unknown" (RFC-044: SNORA_RUNNER_OS exists because GitHub Actions
#   reserves the RUNNER_* namespace and silently ignores any attempt to
#   override RUNNER_OS itself via a step's `env:` block).
#
# Design notes:
# - Uses `cargo clean -p <package>` for per-measurement cold builds so only
#   snora's own crates are rebuilt, not the entire iced transitive closure.
#   This gives a stable, reproducible signal for Snora's own code without
#   penalising CI with a full workspace clean.
# - `cargo clean -p` alone only reaches the dev profile (`-r`/`--release` and
#   `--profile <NAME>` are separate, required flags) — RFC-052. The six
#   measurements below span three profiles (dev, release, release-baseline),
#   so `measure_ms` cleans all three, unconditionally, before every
#   measurement. Cleaning all three rather than passing the one profile a
#   given measurement targets is deliberate: it decouples a measurement from
#   its profile, so a future measurement cannot be added with the wrong one.
#   Before this, four of six measurements silently rode on release/
#   release-baseline artifacts left behind by an earlier measurement in the
#   same run — see docs/src/reference/build-cost-budget.md's RFC-052 note.
# - No CI failure gate; this is a trend signal. See
#   docs/src/reference/build-cost-budget.md for the watch-point policy.
# - Mirrors the binary-size workflow's commit-back pattern on release tags.

set -euo pipefail

VERSION="${1:?Usage: measure-compile-time.sh <version>}"
DATE="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
RUSTC="$(rustc --version | tr ' ' '_')"
# SNORA_RUNNER_OS takes priority: GitHub Actions reserves RUNNER_OS
# itself, so a workflow step cannot override it (RFC-044). Falling back
# to $RUNNER_OS keeps this script's behavior sensible when run outside
# CI's override, e.g. under GitHub's own ambient RUNNER_OS or by hand.
RUNNER_OS="${SNORA_RUNNER_OS:-${RUNNER_OS:-unknown}}"

measure_ms() {
    local name="$1"
    shift
    # Clean snora's own crates across all three profiles measure_ms's callers
    # build with (dev, release, release-baseline) — `cargo clean -p` alone
    # only reaches dev, so a measurement targeting release or
    # release-baseline would otherwise find its artifacts still warm from an
    # earlier measurement in this same run (RFC-052).
    cargo clean -p snora-core -p snora-design -p snora-widgets -p snora 2>/dev/null || true
    cargo clean -p snora-core -p snora-design -p snora-widgets -p snora --release 2>/dev/null || true
    cargo clean -p snora-core -p snora-design -p snora-widgets -p snora --profile release-baseline 2>/dev/null || true
    local start end
    start=$(date +%s%3N)
    "$@" > "/tmp/snora-build-cost-${name}.log" 2>&1
    end=$(date +%s%3N)
    echo $((end - start))
}

check_workspace_ms=$(measure_ms  "check_workspace"    cargo check --workspace --all-features)
build_widgets_ms=$(measure_ms    "build_widgets"      cargo build -p snora-widgets --release)
build_engine_only_ms=$(measure_ms "build_engine_only"  cargo build -p snora --no-default-features --release)
example_hello_ms=$(measure_ms    "example_hello"      cargo build --profile release-baseline -p snora-example-hello)
build_widgets_design_ms=$(measure_ms "build_widgets_design" cargo build -p snora-widgets --features design --release)
# Also clean the workbench binary itself (measure_ms does not include it in
# the package clean), across all three profiles — same defect, same fix
# (RFC-052). example_workbench_ms builds release-baseline, so the dev/release
# cleans below are no-ops for it today, but are included for the same
# decoupling reason as measure_ms's three cleans.
cargo clean -p snora-example-design-workbench 2>/dev/null || true
cargo clean -p snora-example-design-workbench --release 2>/dev/null || true
cargo clean -p snora-example-design-workbench --profile release-baseline 2>/dev/null || true
example_workbench_ms=$(measure_ms "example_workbench"  cargo build --profile release-baseline -p snora-example-design-workbench)

echo "${VERSION},${check_workspace_ms},${build_widgets_ms},${build_engine_only_ms},${example_hello_ms},${build_widgets_design_ms},${example_workbench_ms},${RUSTC},${RUNNER_OS},${DATE}"

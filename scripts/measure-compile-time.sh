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
#   version,check_workspace_ms,build_widgets_ms,build_engine_only_ms,example_hello_ms,rustc,runner_os,date
#
# Design notes:
# - Uses `cargo clean -p <package>` for per-measurement cold builds so only
#   the target package is rebuilt, not the entire iced transitive closure.
#   This gives a stable, reproducible signal for Snora's own code without
#   penalising CI with a full workspace clean.
# - No CI failure gate; this is a trend signal. See
#   docs/src/reference/build-cost-budget.md for the watch-point policy.
# - Mirrors the binary-size workflow's commit-back pattern on release tags.

set -euo pipefail

VERSION="${1:?Usage: measure-compile-time.sh <version>}"
DATE="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
RUSTC="$(rustc --version | tr ' ' '_')"
RUNNER_OS="${RUNNER_OS:-unknown}"

measure_ms() {
    local name="$1"
    shift
    # Clean only the package(s) being measured for a cold build of those crates.
    cargo clean -p snora-core -p snora-widgets -p snora 2>/dev/null || true
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
example_workbench_ms=$(measure_ms "example_workbench"  cargo build --profile release-baseline -p snora-example-design-workbench)

echo "${VERSION},${check_workspace_ms},${build_widgets_ms},${build_engine_only_ms},${example_hello_ms},${build_widgets_design_ms},${example_workbench_ms},${RUSTC},${RUNNER_OS},${DATE}"

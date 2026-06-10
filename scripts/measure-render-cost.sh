#!/usr/bin/env bash
# measure-render-cost.sh <version>
#
# Measures build-time proxies for Snora's layout composition cost.
# Complements measure-compile-time.sh (which measures snora-core,
# snora-widgets, and snora-engine builds) with the workbench example
# as a proxy for "complex AppLayout usage."
#
# Usage:
#   scripts/measure-render-cost.sh 0.16.0
#
# Output (one line, CSV):
#   version,hello_ms,workbench_ms,rustc,runner_os,date
#
# Notes:
# - Uses the release-baseline profile for fast, consistent measurement.
# - Cleans only the two target packages so iced's transitive closure
#   stays cached; the numbers reflect Snora's own code, not iced's.
# - No CI gate — trend signal only. See docs/src/reference/performance-envelope.md.

set -euo pipefail

VERSION="${1:?Usage: measure-render-cost.sh <version>}"
DATE="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
RUSTC="$(rustc --version | tr ' ' '_')"
RUNNER_OS="${RUNNER_OS:-unknown}"

measure_ms() {
    local name="$1"; shift
    cargo clean -p snora-example-hello -p snora-example-workbench 2>/dev/null || true
    local start end
    start=$(date +%s%3N)
    "$@" > "/tmp/snora-render-cost-${name}.log" 2>&1
    end=$(date +%s%3N)
    echo $((end - start))
}

hello_ms=$(measure_ms    "hello"      cargo build --profile release-baseline -p snora-example-hello)
workbench_ms=$(measure_ms "workbench" cargo build --profile release-baseline -p snora-example-workbench)

echo "${VERSION},${hello_ms},${workbench_ms},${RUSTC},${RUNNER_OS},${DATE}"

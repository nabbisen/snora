#!/usr/bin/env bash
#
# append-binary-size-row.sh — append a measurement row to the
# binary-size budget CSV.  This is the only writer of that file
# in normal operation.
#
# Usage:
#
#     scripts/append-binary-size-row.sh <csv-row>
#
# Where <csv-row> is exactly the line produced by
# scripts/measure-binary-size.sh, e.g.
#
#     0.10.0,11500000,11200000,300000,off,2026-04-29
#
# This script is intentionally minimal: it does not parse, sort,
# de-duplicate, or rewrite the file.  Append-only is the entire
# behaviour.  Visualization, drift detection, and history pruning
# are the consumer's concern, not the CSV's.

set -euo pipefail

ROW="${1:?missing CSV row argument}"

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)"
WORKSPACE_DIR="$(cd -- "$SCRIPT_DIR/.." &>/dev/null && pwd)"
CSV_PATH="$WORKSPACE_DIR/docs/src/reference/binary-size-budget/binary-size.csv"

if [[ ! -f "$CSV_PATH" ]]; then
    echo "missing CSV at $CSV_PATH — was the file removed?" >&2
    exit 1
fi

# Sanity-check: the row must have exactly seven comma-separated fields,
# matching the CSV header.  Reject obvious mistakes (empty input,
# wrong column count) before they pollute the file.
field_count=$(awk -F, '{print NF}' <<<"$ROW")
if [[ "$field_count" -ne 9 ]]; then
    echo "row has $field_count fields, expected 9: '$ROW'" >&2
    exit 1
fi

echo "$ROW" >> "$CSV_PATH"
echo "appended to $CSV_PATH:" >&2
echo "  $ROW" >&2

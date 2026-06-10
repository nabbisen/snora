# RFC-012-C — Compile-Time Tracking

Status: Proposed  
Target release: v0.12  
Priority: Medium  
Type: Operational metrics / CI

## 1. Summary

Add lightweight compile-time trend tracking beside the existing binary-size budget. The goal is not to enforce strict
compile-time gates immediately, but to collect release-over-release evidence before compile-time pain becomes invisible.

## 2. Motivation

Snora is intentionally small. Binary size is already tracked, but compile time matters just as much for adoption. GUI
framework users are sensitive to slow iteration loops, and iced itself can be expensive to build cold.

Tracking compile time gives maintainers a signal when optional widgets, features, or dependencies start to harm the
project's “small helper” character.

## 3. Goals

- Record compile-time measurements for representative builds.
- Append release rows similarly to binary-size tracking.
- Avoid strict failure gates in the first version.
- Keep measurements understandable, not statistically perfect.

## 4. Non-Goals

- Do not replace local benchmarking.
- Do not guarantee reproducible performance across GitHub runner generations.
- Do not fail PRs for compile-time regressions initially.
- Do not run expensive matrix benchmarks on every push.

## 5. External Design

Add a docs page or extend existing binary-size reference:

```text
docs/src/reference/build-cost-budget.md
```

or:

```text
docs/src/reference/binary-size-budget.md
```

Preferred: create `build-cost-budget.md` so binary size and compile time are related but not conflated.

CSV file:

```text
docs/src/reference/build-cost-budget/compile-time.csv
```

Header:

```csv
version,check_workspace_ms,build_widgets_ms,build_engine_only_ms,example_hello_ms,rustc,runner_os,date
```

## 6. Internal Design

### 6.1 Measurement Script

Add:

```text
scripts/measure-compile-time.sh
```

Pseudo-implementation:

```bash
#!/usr/bin/env bash
set -euo pipefail

VERSION="${1:?version required}"
DATE="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
RUSTC="$(rustc --version | tr ' ' '_')"
RUNNER_OS="${RUNNER_OS:-unknown}"

measure_ms() {
  local name="$1"
  shift
  cargo clean >/dev/null 2>&1 || true
  local start end
  start=$(date +%s%3N)
  "$@" >/tmp/snora-build-cost-${name}.log 2>&1
  end=$(date +%s%3N)
  echo $((end - start))
}

check_workspace=$(measure_ms check_workspace cargo check --workspace --all-features)
build_widgets=$(measure_ms build_widgets cargo build -p snora-widgets --release)
build_engine_only=$(measure_ms build_engine_only cargo build -p snora --no-default-features --release)
example_hello=$(measure_ms example_hello cargo build --profile release-baseline -p snora-example-hello)

echo "$VERSION,$check_workspace,$build_widgets,$build_engine_only,$example_hello,$RUSTC,$RUNNER_OS,$DATE"
```

This intentionally uses cold builds because cold compile pain is the concern. It is expensive, so run it sparingly.

### 6.2 Workflow

Add `.github/workflows/build-cost.yaml` or extend `binary-size.yaml`.

Recommended: separate workflow to avoid confusing binary-size responsibilities.

Trigger:

```yaml
on:
  push:
    branches: [main]
    tags: ['v*.*.*']
  workflow_dispatch:
```

For PRs, write only a job summary or skip entirely until cost is acceptable.
For tags, append the row and commit back with `[skip ci]`, mirroring binary-size policy.

### 6.3 Threshold Policy

Do not fail initially. Mark thresholds as watch points:

| Metric | Watch point |
|---|---:|
| `cargo build -p snora-widgets --release` cold | 30 seconds on representative developer machine |
| engine-only release build | should remain materially below default/widgets build |
| workspace check all features | track trend only |

The feature-gating criteria already mention a 30-second `snora-widgets` cold build threshold; this RFC operationalizes measurement.

## 7. Documentation Changes

- Add reference page explaining measurement limitations.
- Update feature-gating criteria to point to the CSV.
- Update release process to mention the compile-time row.

## 8. Testing Plan

- Run script locally once.
- Run workflow manually once.
- Confirm CSV row format.
- Confirm commit-back only occurs on release tags if implemented.

## 9. Risks and Mitigations

| Risk | Mitigation |
|---|---|
| GitHub runner variance makes data noisy. | Treat as trend signal, not exact benchmark. |
| Workflow becomes expensive. | Run on tags and manual dispatch first. |
| Cargo cache invalidates measurement. | Use explicit `cargo clean` inside measurement script if cold build is desired. |
| Commit-back recursion. | Use `[skip ci]` as binary-size workflow does. |

## 10. Acceptance Criteria

- Compile-time measurement script exists.
- Documentation explains metrics and limitations.
- At least manual workflow execution produces a valid row.
- Release process mentions compile-time tracking.

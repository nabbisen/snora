# RFC-011-A — Main Rust CI Quality Gate

**Status.** Implemented (v0.11.0)
**Tracks.** Maintenance / CI / release safety. Enforces the documented
local-verification contract on pull requests and `main`.
**Touches.** `.github/workflows/ci.yaml` (new),
`docs/src/contributing/release-process.md`,
`docs/src/contributing/feature-gating-criteria.md`,
optional `docs/src/contributing/ci.md`.

> This is the project's adopted, code-accurate version of the planning-pack
> RFC-011-A. The sections marked **[Design decision]** record choices that
> the planning draft left implicit or open, resolved against the actual
> v0.10.0 tree.

## 1. Summary

Add a main Rust CI workflow that enforces the verification commands already
documented for local development. The existing `docs.yaml` and
`binary-size.yaml` workflows are valuable but do not constitute a
pull-request quality gate for Rust compilation, feature combinations,
engine semantics, or docs-build health.

## 2. Motivation

Snora's central promise is semantic stability: `AppLayout` renders
predictably, feature flags compose predictably, and `snora-core` stays
iced-free. These promises should not depend on a maintainer remembering to
run commands locally. The handoff already fixes the command set
(Part 3 §3.4); the missing work is making it automatic.

## 3. Goals

- Add `.github/workflows/ci.yaml`.
- Enforce, on PR and push to `main`: workspace compile, clippy with
  `-D warnings`, `snora-core` tests, **`snora` engine tests** (so the
  render-semantics tests from RFC-011-D are gated), and engine-only build.
- Add an explicit feature-combination matrix over the public `snora`
  feature surface.
- Build the mdBook as a PR gate (distinct from `docs.yaml`, which deploys).
- Keep the workflow legible at a glance.
- Avoid duplicating the binary-size workflow's responsibilities.

## 4. Non-Goals

- No release-publishing automation.
- Do not replace `binary-size.yaml`.
- Do not test every transitive dependency feature combination.
- Do not create a public `snora-test` crate.
- No GUI/screenshot tests in the first iteration.

## 5. [Design decision] Deviations from the planning draft

1. **Run `cargo test -p snora`, not only `cargo test -p snora-core`.**
   RFC-011-D adds render-semantics tests under `crates/snora/tests/`. Those
   are worthless if CI never runs them. The `rust-quality` job therefore
   runs both crates' test suites. This is the single substantive change to
   the planning-pack YAML and is required for RFC-011-D acceptance
   ("CI runs the tests").
2. **mdBook pinned to `^0.5` `--locked`**, matching the existing
   `docs.yaml` exactly, so the PR gate and the deploy job cannot diverge on
   mdBook behavior.
3. **Feature matrix is code-accurate.** The public optional features of the
   `snora` crate are exactly `widgets` (default), `lucide-icons`,
   `svg-icons` (confirmed in `crates/snora/Cargo.toml`). The matrix below
   enumerates only meaningful public combinations.
4. **`actions/checkout@v4` + `dtolnay/rust-toolchain@stable` +
   `Swatinem/rust-cache@v2`**, consistent with `binary-size.yaml`'s action
   choices so the repo uses one toolchain/cache convention.

## 6. External design

Three jobs:

1. `rust-quality` — workspace check, clippy, `snora-core` tests, `snora`
   tests, engine-only build. The main gate.
2. `feature-matrix` — supported public feature combinations of `snora`.
3. `docs` — mdBook build as a PR gate.

## 7. Internal design

### 7.1 Workflow file

Add `.github/workflows/ci.yaml`:

```yaml
name: CI

on:
  pull_request:
  push:
    branches: [main]
  workflow_dispatch:

concurrency:
  group: ci-${{ github.ref }}
  cancel-in-progress: true

jobs:
  rust-quality:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - name: Check workspace
        run: cargo check --workspace --all-features
      - name: Clippy
        run: cargo clippy --workspace --all-targets --all-features -- -D warnings
      - name: Test core
        run: cargo test -p snora-core
      - name: Test engine (render semantics)
        run: cargo test -p snora
      - name: Check engine-only build
        run: cargo check -p snora --no-default-features

  feature-matrix:
    runs-on: ubuntu-latest
    strategy:
      fail-fast: false
      matrix:
        include:
          - name: default
            args: ""
          - name: no-default-features
            args: "--no-default-features"
          - name: widgets
            args: "--no-default-features --features widgets"
          - name: widgets-lucide-icons
            args: "--no-default-features --features widgets,lucide-icons"
          - name: widgets-svg-icons
            args: "--no-default-features --features widgets,svg-icons"
          - name: all-public-features
            args: "--all-features"
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - name: Check feature combination
        run: cargo check -p snora ${{ matrix.args }}

  docs:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Install mdBook
        run: cargo install mdbook --no-default-features --features search --vers "^0.5" --locked
      - name: Build docs
        run: mdbook build docs
```

### 7.2 Supported feature combinations (normative)

| Combination | Expected result |
|---|---|
| default | Builds with `widgets`. |
| `--no-default-features` | Engine-only build. |
| `--no-default-features --features widgets` | Widget re-exports only. |
| `--no-default-features --features widgets,lucide-icons` | Widgets + Lucide. |
| `--no-default-features --features widgets,svg-icons` | Widgets + SVG. |
| `--all-features` | All public optional features. |

`lucide-icons` / `svg-icons` are **subordinate** to `widgets`: they gate
widget-side rendering re-exports and are not meaningful alone. The matrix
deliberately omits `--features lucide-icons` without `widgets`. This is
documented in `feature-gating-criteria.md` (see RFC-014-D for the v2
tightening).

### 7.3 Cache and compile time

Default `Swatinem/rust-cache@v2`. Do not over-tune cache keys until CI pain
appears. (Compile-time *tracking* is a separate concern — RFC-012-C.)

## 8. Documentation changes

- `docs/src/contributing/release-process.md`: PRs must pass `CI`, `Docs`,
  and `Binary size`; CI is the Rust gate, `docs.yaml` deploys, and
  `binary-size.yaml` tracks size — three distinct responsibilities.
- `docs/src/contributing/feature-gating-criteria.md`: record the supported
  feature-matrix list and the subordinate-feature note.

## 9. Testing plan

The workflow validates itself on first PR. Acceptance:

- passes on a clean branch;
- a deliberate clippy violation fails `rust-quality`;
- a deliberate docs error fails `docs`;
- engine-only build verified;
- `--all-features` verified;
- the engine test step actually executes the RFC-011-D tests.

## 10. Risks and mitigations

| Risk | Mitigation |
|---|---|
| CI slow because iced is heavy. | Rust cache; keep first workflow simple. |
| Feature matrix too broad. | Only public supported combinations. |
| `docs` job duplicates `docs.yaml`. | `docs.yaml` deploys; CI `docs` job is the PR gate. |
| False confidence from core-only tests. | `rust-quality` now also runs `cargo test -p snora` (RFC-011-D). |

## 11. Acceptance criteria

- `.github/workflows/ci.yaml` exists.
- Main Rust commands pass on PR and `main`.
- Feature-matrix job covers the supported public combinations.
- Engine tests are executed by CI.
- Docs explain the CI / docs-deploy / binary-size relationship.
- No release merges while documented local verification commands fail.

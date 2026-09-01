# Developer Handoff — RFC-043 fix the budget measurement methodology

**Governing RFC.** **RFC-043** — find it under `rfcs/done/`, `rfcs/accepted/` or `rfcs/archive/` via the index at [`rfcs/README.md`](../../README.md). *(Not linked by folder: an RFC moves between folders as it ships, and a folder-bearing link here goes stale the moment it does — 14 of them had.)*
**Status.** Inherited from RFC-043 — Implemented (v0.26.0).
**Release target.** Next patch (0.25.4) or folded into v0.26 — owner's call
at release time. Independent of the v0.26 appearance work.
**Implementation units.** One, in five steps. Examples, scripts, CI, docs.

---

## 1. Task title

Make the size probes actually exercise the features they measure, make
build-cost measurement genuinely cold, and stabilise the CSV schema.

## 2. Purpose

RFC-041 fixed the measurement workflows so they finally run on release
tags. The first real data point (0.25.3) showed the instrument was never
calibrated: `widgets_diff_bytes = 0`. Enabling an entire prefab widget
crate measured as costing nothing.

## 3. Background — read first

`rfcs/done/043-measurement-methodology-measures-nothing.md` in full,
especially §Evidence. The short version:

- All three size probes are **byte-identical** and their whole snora
  surface is `AppLayout` + `render`. No widget or design API is ever
  called, so the linker strips the feature code and the diff is ~0.
- The probes' own doc comment demands identical application code "so that
  the diff is purely the marginal cost of the feature" — which inverts the
  intent. Identical code that never *uses* the feature measures the cost
  of compiling-but-not-using it.
- Build-cost numbers are warm: `check_workspace_ms` fell 56150 → 5455
  because `rust-cache` restores `iced` while the clean list covers only
  snora's own packages.

Conventions: English only; `cargo fmt` scoped to crates you touch (~152
hunks of pre-existing workspace drift).

## 4. Change scope

| File | Step |
|---|---|
| `examples/size_probe_widgets/src/main.rs` | 1 |
| `examples/size_probe_design/src/main.rs` | 1 |
| `examples/size_probe_engine/src/main.rs` | 1 (doc comment only) |
| `scripts/measure-compile-time.sh` and/or `.github/workflows/build-cost.yaml` | 2 |
| `scripts/measure-binary-size.sh` / `append-binary-size-row.sh` | 3 (`runner_os`) |
| `docs/src/reference/binary-size-budget.md`, `build-cost-budget.md` | 4 |
| `docs/src/contributing/design-decisions.md` (DEC-14 description) | 1 |
| `.github/workflows/{binary-size,build-cost,unpinned-build}.yaml` | 5 |
| `CHANGELOG.md` | `[Unreleased]` |

## 5. Explicit non-change scope

Do **not**:

- **Delete, edit, or back-fill any CSV row.** RFC-041 N-1 is absolute and
  carries over. The 0.25.3 row stays exactly as it is; it gets annotated,
  not corrected. A re-measured value presented as historical data is a
  fabrication.
- Mark gate 9 satisfied. It re-satisfies only after **two** post-fix rows
  under the corrected methodology exist — which cannot be true when this
  lands.
- Change `feature-gating-criteria.md`'s thresholds. Whether 150 KB is the
  right number is a separate question; this work makes the measurement
  real, not the threshold correct.
- Touch `DEC-11` or the `design` feature default. RFC-043 §"The DEC-11
  implication" is explicitly deferred to the owner, on evidence.
- Touch any crate source under `crates/`.

## 6. Required implementation

### Step 1 — Make the probes exercise their features

The probes stay **as similar as possible** — the diff must still isolate
the feature, not application logic. The change is that each probe adds a
*minimal, representative use* of the feature it is named for.

| Probe | Should render |
|---|---|
| `size_probe_engine` | unchanged — baseline. Only the doc comment is corrected. |
| `size_probe_widgets` | the baseline body **plus** a representative `snora::widget::*` call — e.g. `app_header` and `app_side_bar` wired into the `AppLayout` |
| `size_probe_design` | the widgets set **plus** a representative `snora::design::*` use — e.g. one `design::button::primary` and one `design::style::container::card_surface` against `Tokens::light()` |

Keep the additions small and obviously representative. The number this
produces answers "what does a typical adopter of this feature pay", which
is what the budgets were created to track.

**Rewrite the doc comment in all three probes.** The current one is the
trap that produced this RFC:

> the point is that all three probes … contain the same application code
> so that `widgets_diff` and `design_diff` are purely the marginal cost of
> the respective feature

Replace it with the corrected intent: probes share a common baseline body
and differ **only** by a minimal use of the feature under test, so the
diff is the marginal cost of *using* that feature. Say explicitly that
identical, feature-unused code measures ~0 because unused code is stripped
at link time — so the next person does not "simplify" it back.

Update DEC-14's description in `docs/src/contributing/design-decisions.md`
to match. Its status and reconsideration trigger stay as they are.

### Step 2 — Make build-cost genuinely cold

Either disable dependency caching in `build-cost.yaml`, or clear the whole
target directory before the timed section — not just snora's own packages.

Expect run times to rise from ~1 minute to several. **That is correct**,
not a regression: RFC-041's rationale for keeping measurement off the PR
path already accounts for it. If you take the cache-disabling route, say
so in the review request so the jump in CI time is not mistaken for a
fault later.

Do **not** make `binary-size.yaml` cold as well unless measurement
correctness requires it — binary size is a property of the linked output,
not of build freshness. If you believe it does require it, say why rather
than doing it silently.

### Step 3 — Stabilise `runner_os`

The 0.25.3 row reads `Linux`; every prior row reads `ubuntu-latest`. Emit
a consistent value so rows group. Match the historical spelling
(`ubuntu-latest`) rather than rewriting history to match the new one.

### Step 4 — Annotate the discontinuity

In both budget docs, record that **0.25.3 is simultaneously the first row
produced by the fixed workflows and the last row produced by the broken
methodology**, and is not comparable to rows on either side. Keep it
short; the detail lives in RFC-041 and RFC-043.

### Step 5 — `actions/checkout@v4` → `@v6`

Folded in here by the architect (it was raised as a deferred follow-up
after the 0.25.3 release, and this is the natural handoff for it — it is
easily dropped if the owner would rather it stood alone).

`binary-size.yaml`, `build-cost.yaml` and `unpinned-build.yaml` pin
`actions/checkout@v4`, which targets the deprecated Node 20 and is being
force-run on Node 24. `ci.yaml` and `docs.yaml` already use `@v6`. Bring
the three into line.

This was deliberately **not** done before the 0.25.3 tag, so that the
first-ever measurement run happened on a verified configuration. That run
has now succeeded, so the reason to wait is gone. Check whether
`actions/upload-artifact` and `stefanzweifel/git-auto-commit-action` carry
the same warning and report — but do not bump them in this change without
saying so.

## 7. Required tests

```bash
cargo check --workspace --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
mdbook build docs && mdbook test docs
```

Plus, and this is the point of the whole change:

```bash
bash scripts/measure-binary-size.sh     # or however the workflow invokes it
```

**Report the actual numbers.** `widgets_diff_bytes` and
`design_diff_bytes` must be non-trivial. If either is still ~0 after Step
1, the fix did not work — stop and escalate rather than shipping a second
broken methodology.

Validate both workflows' YAML, and dispatch `unpinned-build` once
(`workflow_dispatch`) to confirm the `@v6` bump did not break it.

## 8. Acceptance criteria

RFC-043 §Acceptance criteria 1–6:

1. Each probe exercises its feature; sources are no longer byte-identical;
   doc comments explain why.
2. `widgets_diff_bytes` / `design_diff_bytes` are non-trivial and
   reproducible.
3. Build-cost numbers are cold and comparable in magnitude to the
   pre-cache era (tens of seconds, not hundreds of milliseconds).
4. `runner_os` is stable across rows.
5. Budget docs record the 0.25.3 discontinuity; **no row deleted or
   edited**.
6. Gate 9 remains ⬜.

Plus: the three workflows use `actions/checkout@v6` and still run.

## 9. Prohibited shortcuts

- Do not back-fill or "correct" the 0.25.3 row.
- Do not tune the probes until the diff looks like a number you expected.
  Measure, then report whatever it says.
- Do not mark gate 9 satisfied.
- Do not adjust `feature-gating-criteria.md` thresholds to fit the new
  numbers.

## 10. Compatibility and security

Neither is affected: examples, scripts, CI and docs only; no crate source,
public API, or feature flag changes. State this explicitly.

## 11. Required evidence

- Diffs of the three probe sources and their doc comments.
- **The measured numbers**, before and after Step 1, side by side.
- Build-cost timings showing cold behaviour.
- `git diff --stat -- 'docs/src/reference/**/*.csv'` — must be empty.
- `git diff --stat -- crates/` — must be empty.
- Workflow diffs and a successful `unpinned-build` dispatch.

## 12. Required review-request format

Per workflow policy §9.2 and the packaging convention: `README.md` entry
point, full `review-request.md`, `evidence/`, under
`.git-exclude/review-request/043-measurement-methodology-measures-nothing/`.

**Requested review focus:** whether the probes' feature use is genuinely
*representative* rather than arbitrary — an unrepresentative probe
measures a number as meaningless as zero, just less obviously.

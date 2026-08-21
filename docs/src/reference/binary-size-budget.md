# Binary size budget

**Latest values and full history:**
[binary-size-budget/binary-size.csv](binary-size-budget/binary-size.csv)

Released values are appended to that CSV automatically on every
release tag push by the
[`binary-size`](https://github.com/nabbisen/snora/actions/workflows/binary-size.yaml)
GitHub Actions workflow.

## Why this page exists

Snora targets desktop GUI applications, where executable size has
real consequences for distribution — installers, app stores,
auto-updaters, end-user disk space. Adding a feature to the
framework should never quietly cost users megabytes of bloat.

This page tracks the stripped binary size of three probe apps
(`examples/size_probe_engine`, `size_probe_widgets`, `size_probe_design`)
at every release tag. All three share a common baseline application, and
each adds a minimal, representative *use* of the feature it measures —
`size_probe_widgets` calls `app_header`/`app_side_bar`,
`size_probe_design` additionally calls a `design::button` and a
`design::style::container` helper (see RFC-043). The size difference
between two consecutive probes is the **marginal cost** of actually
*adopting* that feature, not merely compiling it in.

Tracking the number across releases gives us **drift detection**:
if 0.10 → 0.11 grows the binary by 200 KB without a corresponding
new feature, that is a regression to investigate before publish,
not a surprise users find six months later.

The threshold values that govern when to act on the data live in
[`feature-gating-criteria.md`](../contributing/feature-gating-criteria.md)
indicator (2). This page is the data; that page is the policy.

### Data integrity note (rustc 1.97.1 → 1.98.0, at 0.39.2)

**The runner's compiler changed between 0.39.1 and 0.39.2**, and it moved the
derived columns far more than snora did — snora changed nothing that ships. The
release added one `#[cfg(test)]` assertion and edited documentation;
`git diff 0.39.1 0.39.2 -- crates/` is a single test file.

| column | 0.39.1 | 0.39.2 | move |
|---|---|---|---|
| `rustc` | `1.97.1 (8bab26f4f 2026-07-14)` | **`1.98.0 (88d9e12ae 2026-08-18)`** | — |
| `engine_bytes` | 15,690,704 | 15,689,304 | −1,400 B (−0.0089%) |
| `widgets_bytes` | 15,737,168 | 15,738,072 | +904 B (+0.0057%) |
| **`widgets_diff_bytes`** | 46,464 | **48,768** | **+2,304 B (+4.96%)** |
| **`design_diff_bytes`** | 3,840 | **1,664** | **−2,176 B (−56.7%)** |

**Read the absolute and derived columns differently here.** The two absolute
totals moved by under 0.01% — the compiler produced almost the same binary. The
two *differences* moved by 5% and 57%, because a difference between two large
near-equal numbers amplifies any redistribution between them. **Nothing grew;
the split moved.**

**This is a methodology discontinuity, the fourth** (after RFC-043, RFC-044,
RFC-052). Per RFC-041 N-1 no historical row is edited or back-filled — it is
recorded, not repaired. **`widgets_diff_bytes` and `design_diff_bytes` are not
comparable across the 0.39.1 / 0.39.2 boundary.**

### The band did not carry over — measured, at 0.39.3

**0.39.3 was cut to produce a second row on 1.98.0**, with no shipped code in
it, precisely so the toolchain's own variance could be read on its own. It was:

| | 0.39.2 | 0.39.3 (zero code) | move |
|---|---|---|---|
| `engine_bytes` | 15,689,304 | 15,689,304 | **0** |
| `design_bytes` | 15,739,736 | 15,739,736 | **0** |
| `widgets_bytes` | 15,738,072 | 15,737,816 | −256 B |
| **`widgets_diff_bytes`** | 48,768 | 48,512 | **−256 B** |
| `design_diff_bytes` | 1,664 | 1,920 | +256 B |

**−256 B on a release that shipped nothing — twice the ±128 B amplitude
established on 1.97.1.** The band did **not** carry across the compiler change,
and one observation is all we have on 1.98.0, so treat **256 B as the floor of
what is currently distinguishable**, not as a new band.

Note what did *not* move: `engine_bytes` and `design_bytes` are **byte-identical**
across the two rows. The variance on this compiler is confined to the widgets
binary, which is also the numerator of the column the budget is judged on.

**The general lesson, which cost two releases to learn:** a noise band belongs to
a toolchain, not to a project. Every figure quoted in this file is implicitly
suffixed *"on the compiler it was measured with."*

**It also puts the ±128 B noise band in its place.** That band was established
on three zero-code releases *on one compiler*, and it holds only within a
toolchain. A rustc change is not noise and is not signal about snora — it is a
third category, and the `rustc` column exists so it can be told apart from both.
Without that column this row would read as a 5% regression on a documentation
release.


## How this is updated

The flow is split between automation and human discipline:

- **CI runs the measurement on every push to `main` and every
  pull request.** Results land in the workflow's job summary and a
  30-day artifact. *No file is modified for these runs.* This
  gives developers visibility into accidental size regressions
  without polluting the repository history.

- **CI appends one row to the budget CSV on every release tag**
  (the project's tags carry no `v` prefix, e.g. `0.25.3`), then commits
  the change back to `main` with `[skip ci]`. This is the only path by
  which the CSV grows. One release, one row.

- **Humans never edit the CSV directly.** Manual edits would
  shadow the bot's commits and could be lost on the next release;
  there is also no scenario where the value being recorded is
  better measured by hand than by the CI's own scripts.

The script that produces a row is
[`scripts/measure-binary-size.sh`](https://github.com/nabbisen/snora/blob/main/scripts/measure-binary-size.sh).
It is intentionally simple — `cargo build` × 2, `strip --strip-all`
× 2, `stat -c '%s'` × 2 — so that the values are reproducible
locally:

```text
scripts/measure-binary-size.sh 0.10.0
```

## Reading the numbers

Each CSV row records:

| Column | Meaning |
|---|---|
| `version` | snora version this row is for, e.g. `0.25.0`. |
| `engine_bytes` | Stripped size of `snora-size-probe-engine` (`--no-default-features`). |
| `widgets_bytes` | Stripped size of `snora-size-probe-widgets` (default features). |
| `widgets_diff_bytes` | `widgets_bytes − engine_bytes`. Marginal binary cost of `snora-widgets`. |
| `design_bytes` | Stripped size of `snora-size-probe-design` (`features = ["widgets", "design"]`). |
| `design_diff_bytes` | `design_bytes − widgets_bytes`. Marginal binary cost of `snora-design`. |
| `rustc` | Rust toolchain version, e.g. `rustc_1.96.0_(ac68faa20_2026-05-25)`. |
| `runner_os` | CI runner OS, e.g. `ubuntu-latest`. |
| `date` | UTC date of the measurement (`YYYY-MM-DD`). |

All three probes share a common baseline application and differ **only**
by a minimal, representative use of the feature under test (see
`examples/size_probe_*/src/main.rs`) — through 0.25.3 the probes were
byte-identical instead, which measured the cost of compiling a feature in
but never calling it; RFC-043 corrected this (see the discontinuity note
below). Rows before v0.25 carry `N/A` because the probe crates did not
exist and earlier measurements used different methodology.

### Data integrity note (RFC-041)

Every row through 0.25.2 carries `N/A` in all measurement columns. The
workflow's release-tag trigger matched `v*.*.*`, but the project's tags
carry no `v` prefix, so the append-on-tag step never fired on any of the
project's 38 release tags — not because of the `N/A` schema migration
described above, but because no CI run ever executed the append at all.
This was fixed in 0.25.3 (RFC-041). Rows before 0.25.3 therefore predate
the fix and were not produced by the tag automation. Additionally, the
series spans the 0.25.2 `resolver = "2"` → `"3"` workspace change; at the
time of the 0.25.3 measurement, `Cargo.lock` was not yet committed for
any *prior* release, so dependency resolution was not pinned between the
CI runs that produced pre-0.25.3 rows — pre-0.25.3 and post-0.25.3 rows
are **not comparable** even where both are non-`N/A`. `Cargo.lock` is
committed as of 0.25.3 itself (RFC-042), so resolution *is* pinned for
0.25.3 and every release after it — this caveat does not apply going
forward.

### Data integrity note (RFC-043)

The 0.25.3 row is doubly transitional: it is simultaneously the **first**
row produced by the fixed tag-automation (RFC-041) and the **last** row
produced by the broken probe methodology (byte-identical probes that
never called the feature they measured, described above). Its
`widgets_diff_bytes` (`0`) and `design_diff_bytes` (`128`) reflect that
defect, not a real marginal cost, and are **not comparable** to any row
from 0.25.4 onward, which uses the corrected feature-exercising probes.
Do not treat the 0.25.3 diffs as a baseline for drift detection.

The 150 KB threshold from
[`feature-gating-criteria.md`](../contributing/feature-gating-criteria.md)
indicator (2) applies to `diff_bytes`. If a release crosses that
threshold, the criteria document specifies what to do (it does
not unilaterally trigger a per-widget feature split — see the
document for the full rule).

### Data integrity note (RFC-044)

`runner_os` reads `Linux` for both the 0.25.3 and 0.26.0 rows, not
`ubuntu-latest` like every row before them. RFC-043 attempted to
stabilize this column via `env: RUNNER_OS: ubuntu-latest` on the
measuring step; that override is silently ignored, because GitHub
Actions reserves the entire `RUNNER_*` variable namespace and does not
permit overwriting it. The workflow file looked correct on inspection —
present, well-commented — and two review passes confirmed it; only the
emitted row revealed the defect, and no post-fix row existed until the
0.26.0 tag. Fixed in RFC-044 by routing the value through
`SNORA_RUNNER_OS`, a name GitHub does not reserve. The two `Linux` rows
are not edited or back-filled — they stand as a recorded discontinuity,
per the append-only policy (RFC-041 N-1). This criterion is not closed by
the fix landing; it closes only when the next tagged release's row is
confirmed to read `ubuntu-latest`.

### Build profile

Measurements use the `[profile.release-baseline]` Cargo profile, which
inherits from `[profile.release]` but with `lto = false` and
`codegen-units = 16`. This keeps CI build time to 2–4 minutes per
configuration rather than 10–20 minutes for a full LTO build.

The cost is that `release-baseline` binaries are **20–40% larger than what
a user actually ships** with the default `[profile.release]`. That is
acceptable for the budget's purpose: every row is measured under the same
profile, so the diff between consecutive rows accurately reflects the change
in the framework's contribution. The `rustc` and `runner_os` columns allow
filtering out measurements taken under different toolchains or platforms.

## Frequently checked questions

**Where does this page get edited by humans?** Above this line.
Everything below the CSV link at the top of the page is prose,
maintained by hand. Inside `binary-size-budget/binary-size.csv`,
nothing is maintained by hand — it is bot-only.

**Why no chart on this page?** Plotting was deferred. The CSV is
the data store; visualization tooling can be added later (Plotly,
a `gh-pages` chart, an external dashboard) without changing how
the data flows. Keeping the data store separable from the
visualization is exactly why the CSV lives in its own subfolder.

**Why is the CSV empty / missing my release?** The first *real* row is
expected starting with the 0.25.3 tag (RFC-041 fixed the tag-pattern
mismatch that silently prevented every prior release from appending one —
see the data integrity note above). If a release at or after 0.25.3 ships
and no row appears, don't assume the workflow was green — a workflow that
never triggers reports nothing rather than failing. Confirm the row exists
in the CSV directly (`git show main:docs/src/reference/binary-size-budget/binary-size.csv | tail -1`)
before treating the release as clean; if it's missing, check the Actions
tab for whether `binary-size` triggered on the tag at all.

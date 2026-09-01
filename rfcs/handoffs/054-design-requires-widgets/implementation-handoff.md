# Developer Handoff — RFC-054 `design` requires `widgets`

**Governing RFC.** **RFC-054** — find it under `rfcs/done/`, `rfcs/accepted/` or `rfcs/archive/` via the index at [`rfcs/README.md`](../../README.md). *(Not linked by folder: an RFC moves between folders as it ships, and a folder-bearing link here goes stale the moment it does — 14 of them had.)*
**Status.** Inherited from RFC-054 — Closed (v0.32.0).
**Release target.** None yet — see §1.
**Implementation units.** One. **This is an investigation, not an
implementation.**

---

## 1. What this task is

**Do not implement options A, B or C.** RFC-054 recommends none of them
deliberately, and the owner accepted the RFC as an investigation.

Your deliverable is **evidence and a recommendation**, not a diff to the
feature graph:

1. an answer to Q-1 — is `card_raised` a widget-layer thing at all? — argued
   from layering, not only mechanics;
2. an answer to Q-3 — what a design-path consumer would **actually** save,
   measured, not assumed from `widgets_diff_bytes`;
3. a recommendation among A / B / C, with the reasoning that data supports.

The RFC's acceptance criteria 3 and 4 (record the decision; document the
coupling in `feature-flags.md`) follow the owner's choice. **Criterion 4 is
required even if the outcome is "decline"** — the consumer who raised this
found the coupling by reading `Cargo.toml`, which is where it should not have
had to be found.

A spike is expected and welcome. **Revert it before you report.**

## 2. Background — read first

- `rfcs/done/054-design-requires-widgets.md` in full.
- `rfcs/done/039-engine-surface-styling.md` §"The dialog card" — why
  `card_raised` is reused rather than reimplemented. That decision constrains
  every option.
- `docs/src/reference/binary-size-budget.md` — how the three probes work.
- `docs/src/contributing/feature-gating-criteria.md` indicator 2 — the
  existing test for whether a size cost justifies a gate.

## 3. The trap

`crates/snora/src/design/render.rs` touches `snora-widgets` in **two** places,
and one of them is invisible to an import scan:

```rust,ignore
use snora_widgets::design::style::color::to_iced_color;          // in the import list
…
let mut style = snora_widgets::design::style::container::card_raised(tokens);   // line 173, fully qualified
```

An earlier reading of this problem counted only the `use` imports, found one
helper, and concluded the coupling was trivial. **Grep the file body, not its
imports.** Verify there is no third call before you scope anything.

## 4. Q-1 — the layering question, and do it first

`card_raised` returns `iced::widget::container::Style`. It is a **style**, not
a widget: no `Element`, no layout, no message. `to_iced_color` is a two-field
type conversion.

If the style layer sits conceptually **below** the widget layer, then both
functions are in the wrong crate today, and options A and B are much smaller
than the feature line suggests — the current placement is the accident, not
the coupling.

If the style layer is genuinely part of the widget layer — because it exists
to serve the prefab widgets and its API is shaped by them — then the engine
surface reaching into it is the anomaly, and RFC-039's reuse decision deserves
re-examination rather than the feature graph.

**Answer this before pricing anything.** It determines which option is even
coherent, and it is a design question you can settle by reading
`crates/snora-widgets/src/design/style/` and asking what else depends on it.

## 5. Q-3 — measure the real saving

**`widgets_diff_bytes` (46,336 at v0.30.0) is not the answer** and must not be
reported as one. A design-path consumer would still retain whatever
`design::render`'s two calls pull in, even after a split.

The machinery already exists. `scripts/measure-binary-size.sh` builds three
probes that share a common baseline application and differ only by feature:

| Probe | snora dependency |
|---|---|
| `size_probe_engine` | `default-features = false` |
| `size_probe_widgets` | workspace default |
| `size_probe_design` | `features = ["widgets", "design"]` |

**The probe that would answer Q-3 does not exist**, because the configuration
it measures is unreachable — which is the whole finding. So:

1. Spike the minimal change that makes "design without widgets" compile — most
   likely relocating the two functions per §4's answer.
2. Add a fourth probe on that spike, matching the existing three's baseline
   application exactly, so the diff is attributable.
3. Measure it against `size_probe_engine`.
4. **Revert the spike.** Report the number and the diff you used to get it; do
   not leave either in the tree.

If the spike turns out to be substantially larger than relocating two
functions, **stop and report that instead** — a much larger change is itself
the answer to whether this is worth doing, and it makes option C stronger.

## 6. Explicit non-change scope

Do **not**:

- **Implement A, B or C.** Recommendation only (§1).
- **Duplicate the card mapping.** RFC-039's reuse decision stands under every
  option; a spike that copies `card_raised` into the engine surface is
  measuring the wrong thing.
- **Move anything into `snora-design`.** It is iced-free by hard constraint
  and both functions return `iced` types. Not negotiable, not a finding.
- **Change `snora::design`'s public surface**, or `snora-widgets`'s.
- **Leave the spike, the fourth probe, or a modified `Cargo.toml` in the
  tree.**
- Touch `binary-size.csv`. No row is appended for a spike measurement.

## 7. Required tests

There is no code deliverable, so the usual gates apply only to the spike while
it exists. Before reporting, confirm the tree is back to its committed state:

```bash
git status --porcelain     # must be empty apart from your review-request files
cargo check --workspace --all-features
```

If you added a probe crate, confirm it is gone and that
`cargo check --workspace --all-features` still passes without it.

## 8. Acceptance criteria

1. Q-1 answered, argued from layering.
2. Q-3 answered with a **measured** figure and the method stated, or an
   explicit "the spike was larger than expected, here is why" (§5).
3. A recommendation among A / B / C, with reasoning tied to 1 and 2.
4. The spike reverted; tree clean.
5. Confirmation of how many `snora-widgets` call sites exist in
   `design/render.rs` — verified against the file body, not its imports (§3).

Criteria 3 and 4 of the **RFC** (record the decision; document in
`feature-flags.md`) are the owner's follow-up, not yours, and depend on which
option is chosen.

## 9. Prohibited shortcuts

- Do not report `widgets_diff_bytes` as the saving (§5).
- Do not answer Q-1 by measuring. It is a design question; the measurement
  answers Q-3.
- Do not recommend an option you did not price.
- Do not skip the recommendation because the RFC declines to make one. The RFC
  withholds it because it lacks your data, not because none should be offered.

## 10. Required evidence

- The `design/render.rs` call-site audit (§3, criterion 5).
- Your Q-1 reasoning, and what you read to reach it.
- The spike diff, the fourth probe's source, and the measured comparison.
- Proof the spike is reverted — `git status --porcelain` and a clean
  `cargo check --workspace --all-features`.

## 11. Required review-request format

Per workflow policy §9.2 and the packaging convention: `README.md` entry
point, full `review-request.md`, `evidence/`, under
`.git-exclude/review-request/054-design-requires-widgets/`. **State the single
entry-point path to hand to the reviewer** in the completion summary.

**Requested review focus:** Q-1. The measurement will be what it is; whether
the style layer belongs below the widget layer is the judgement that decides
whether this is a small correction or a crate-boundary change, and it is the
part I would most like a second opinion on.

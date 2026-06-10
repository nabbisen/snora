# RFC-013-C — Tooltip Vocabulary and Persistent Toast Helper

**Status.** Implemented (v0.13.0)
**remain deferred**: trigger evidence is absent as of v0.12.0. This
version records the evidence check and updates the trigger criteria.

**Tracks.** Small API ergonomics (watch list).
**Touches.** `docs/src/contributing/design-decisions.md` (evidence note),
no code.

## 1. Evidence check as of v0.12.0

### Candidate A — Tooltip Vocabulary

Trigger: second consumer beyond `SideBarItem.tooltip: String`.

Survey of v0.12.0 codebase tooltip-like fields:

| Type | Field | Type |
|---|---|---|
| `SideBarItem<M, V>` | `tooltip: String` | only one |

The workbench uses `SideBarItem` tooltips but does not add a second
consumer type. **Trigger not met.**

### Candidate B — Persistent Toast Helper

Trigger: at least two examples/apps repeating `Toast::new(...).persistent()`.

Survey of v0.12.0 examples:

| File | Usage |
|---|---|
| `examples/workbench/src/main.rs` | Uses `Toast::new(…)` but does not call `.persistent()` |

No example calls `.persistent()`. **Trigger not met.**

## 2. Decision: both remain deferred

No implementation in v0.13.0. Watch for the trigger conditions at each
release. Update this RFC when the trigger is met.

## 3. Trigger conditions (restated for clarity)

**Tooltip vocabulary trigger:** a second public `snora-core` or
`snora-widgets` type gains a tooltip-like field. When that happens,
extract `Tooltip { text: String, side: Edge }` from `snora-core`, update
`SideBarItem`, and ship with ABDD checklist completed.

**Persistent toast helper trigger:** two separate examples or downstream
apps contain `Toast::new(…).persistent()` — not counting the same call
site copied. At that point add `Toast::persistent_ack(…)` as a named
constructor (Option 1 from planning draft) with a doctest.

## 4. Acceptance criteria

- No code change.
- Evidence check is recorded here.
- design-decisions.md references this RFC for the "defer tooltip" decision.

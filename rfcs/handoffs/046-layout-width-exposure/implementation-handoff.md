# Developer Handoff — RFC-046 layout width exposure

**Governing RFC.** **RFC-046** — find it under `rfcs/done/`, `rfcs/accepted/` or `rfcs/archive/` via the index at [`rfcs/README.md`](../../README.md). *(Not linked by folder: an RFC moves between folders as it ships, and a folder-bearing link here goes stale the moment it does — 14 of them had.)*
**Status.** Inherited from RFC-046 — Implemented (v0.28.0).
**Release target.** 0.28.0, alongside RFC-047.
**Implementation units.** One. **Read §5 before starting — this and
RFC-047 both touch `render.rs`.**

---

## 1. Task title

Expose the layout's available width to the application, via a sibling
render entry point. Add no breakpoint behaviour.

## 2. Purpose

snora observes window size **nowhere**. Every consumer wanting breakpoints
must write window observation from scratch, and the downstream report is
that a team may not notice it is missing until someone audits — their own
responsive-design RFC had gone unimplemented, and neither their application
nor snora observed size anywhere.

## 3. Background — read first

- `rfcs/done/046-layout-width-exposure.md` in full, especially
  §"Why exposure and not behaviour".
- `rfcs/done/039-engine-surface-styling.md` — establishes the sibling
  entry-point pattern this follows, and the `render_semantics`-unmodified
  invariant.

Conventions: English only; Rust 2018+ modules, tests in `foo/tests.rs`;
`cargo fmt` **scoped to `snora`** (~152 hunks of pre-existing drift).

## 4. The trap that will cost you an hour

**Searching iced's source for "responsive" finds nothing.** There is no
`pub use ... responsive` in the `iced` crate, no mention in its `lib.rs`,
and a repository-wide grep returns only `README.md` and `CHANGELOG.md`.

It is nevertheless reachable:

```rust
use iced::widget::Responsive;   // compiles — verified
```

`iced::widget` does `pub use iced_widget::*`, and `iced_widget` exports
`Responsive` unconditionally. The glob carries it; nothing names it.

This was confirmed **by compiling** against `iced = "0.14"` with snora's
feature set, not by reading. Do not conclude from a failed grep that you
need a new dependency or the `lazy` feature — **you need neither.**

`Responsive` holds `Box<dyn Fn(Size) -> Element>`.

## 5. Ordering with RFC-047 — read before starting

RFC-047 (stable identifiers) also modifies `crates/snora/src/render.rs`.
The two are **semantically independent** — one wraps composition, the other
labels surfaces inside it — but they will conflict textually.

Do **not** run them in parallel on the same branch. Either:

- land RFC-047 first (it adds labels inside the existing composition, which
  is the smaller edit), then this; or
- land this first and rebase RFC-047.

Whichever lands second **re-runs `render_semantics`** and confirms it still
passes unmodified. Coordinate with the architect if both are in flight.

## 6. Change scope

| File | Purpose |
|---|---|
| `crates/snora/src/responsive.rs` (new; name at your discretion) | the entry point |
| `crates/snora/src/responsive/tests.rs` (new) | tests |
| `crates/snora/src/lib.rs` | declare + re-export |
| `crates/snora/src/render.rs` | share composition if needed — see §7 |
| `docs/src/guides/` | a guide page |
| `examples/` | a runnable example |
| `CHANGELOG.md` | `[Unreleased]` **Added** |

## 7. Required implementation

### Step 1 — The entry point

```rust,ignore
pub fn responsive_render<'a, Message, F>(build: F) -> Element<'a, Message>
where
    F: Fn(f32) -> AppLayout<Element<'a, Message>, Message> + 'a,
    Message: Clone + 'a;
```

The application supplies a closure building its layout from the available
width; snora renders the result through the **existing** z-stack.

**Do not duplicate the layer composition.** RFC-039 already extracted a
shared path for exactly this reason — reuse it. Two copies of the z-stack
will diverge, and the layer order is a documented contract.

**This is not design-gated.** It is engine capability and belongs in the
default surface next to `render`, not behind `design`.

### Step 2 — Decide and flag: `f32` or `Size`

Width is what was asked for and is the narrower contract. Height costs
nothing extra and may be equally useful. **Pick one, and say why in the
review request** — do not decide silently. This is RFC-046 Q-2.

### Step 3 — Guide and example

The guide must state plainly that **thresholds are the application's
decision** and snora deliberately does not prescribe them. Otherwise
readers will expect breakpoints and find only a number.

The example should be runnable and demonstrate a threshold the *application*
chooses — e.g. dropping the sidebar below some width the example picks for
itself.

## 8. Explicit non-change scope

Do **not**:

- Add a `Breakpoint` enum, thresholds, or any adaptive behaviour. That is
  deliberately deferred pending evidence about which thresholds consumers
  converge on — evidence that requires shipping this first.
- Add a width field to `AppLayout`. It lives in `snora-core`, which is
  iced-free and has no dependencies; its contract should not gain a
  rendering-time input. Same reasoning as RFC-039.
- Add any dependency. `Responsive` is reachable through `iced` (§4).
- Feature-gate it.
- Change `snora::render`'s signature or output.

## 9. The invariant

`crates/snora/tests/render_semantics.rs` must pass **unmodified**. If a
test needs changing, composition has been altered — stop and escalate
rather than adjusting the test.

## 10. Required tests

```bash
cargo fmt --check -p snora
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test -p snora --lib --all-features
cargo test -p snora --test render_semantics     # MUST pass unmodified
cargo check -p snora --no-default-features      # NOT design-gated — must pass
cargo check -p snora --no-default-features --features widgets
mdbook build docs && mdbook test docs
```

New tests:

| Test | Assertion |
|---|---|
| Width reaches the closure | A layout built through `responsive_render` receives a plausible width |
| Composition matches | The z-stack produced equals `render`'s for an equivalent layout |

**On coverage honesty:** if `iced_test`'s simulator cannot exercise
available width, **say so** rather than asserting coverage that does not
exist. A runnable example that can be observed is an acceptable substitute
for that one property — but the review request must state which properties
are test-covered and which are example-demonstrated.

## 11. Acceptance criteria

RFC-046 §Acceptance criteria 1–5:

1. `snora::responsive_render` exists in the **default** surface and is
   documented.
2. `snora::render` unchanged; `render_semantics` passes unmodified.
3. No new dependency; no `AppLayout` field.
4. No breakpoint vocabulary or adaptive behaviour.
5. A guide page and runnable example exist, and state that thresholds are
   the application's decision.

## 12. Prohibited shortcuts

- Do not duplicate the z-stack composition.
- Do not add a `Breakpoint` type "since it is obvious what the values
  should be." It is not — that is the open question this defers.
- Do not modify a render-semantics test to make it pass.
- Do not add `iced_widget` as a direct dependency (§4).

## 13. Compatibility and security

**Compatibility.** Purely additive. Applications not calling the new entry
point are unaffected. State this explicitly.

**Security.** No new data flow, dependency, or integration.

## 14. Required evidence

- The new module in full, and any `render.rs` diff.
- Test output, with the coverage-honesty statement from §10.
- `render_semantics` output plus `git diff --stat -- crates/snora/tests/`
  showing it is **empty**.
- Both `--no-default-features` check results.
- Your `f32`-vs-`Size` decision and reasoning.

## 15. Required review-request format

Per workflow policy §9.2 and the packaging convention: `README.md` entry
point, full `review-request.md`, `evidence/`, under
`.git-exclude/review-request/046-layout-width-exposure/`. **State the
single entry-point path to hand to the reviewer** in the completion
summary.

**Requested review focus:** whether the entry point earns its place beside
`render` and `design::render` — three is a lot for a framework whose value
is a small readable surface, and RFC-046 Q-1 flags that cost deliberately.

# Developer Handoff — RFC-039 engine surfaces: dialog card and modal dim

**Governing RFC.** **RFC-039** — find it under `rfcs/done/`, `rfcs/accepted/` or `rfcs/archive/` via the index at [`rfcs/README.md`](../../README.md). *(Not linked by folder: an RFC moves between folders as it ships, and a folder-bearing link here goes stale the moment it does — 14 of them had.)*
**Status.** Inherited from RFC-039 — Implemented (v0.27.0).
**Release target.** v0.27.0, alongside RFC-040.
**Implementation units.** One. Independent of RFC-040 — they can run in
parallel.

---

## 1. Task title

Add `snora::design::render(layout, &tokens)`, giving the dialog a real card
and the modal dim a token-derived colour.

## 2. Purpose

The dialog is the most visually prominent thing snora draws, and it draws
nothing:

```rust
// crates/snora/src/overlay/dialog.rs:14
center(dialog.content).into()
```

No background, padding, radius or border — bare content on a fixed grey
wash (`render.rs:192,210`, `rgba(0,0,0,0.4)`). This is the surface that
most directly produced the complaint that a snora application is "not kind
to see."

## 3. Background — read first

- `rfcs/done/039-engine-surface-styling.md` in full.
- `rfcs/done/037-coherent-defaults-positioning.md` — the **gating
  invariant** this must preserve.
- `rfcs/done/036-design-surface-freeze-and-additive-covenant.md` — the
  covenant. RFC-039 is its first real test; read §"The covenant bites here".

Conventions: English only; Rust 2018+ modules, tests in `foo/tests.rs`;
`cargo fmt` **scoped to the crates you touch** (~152 hunks of pre-existing
workspace drift).

## 4. The owner's decision you are implementing

**Work within the covenant. Do not add token roles.**

Two things this work wants do not exist in the frozen surface: a
scrim/overlay colour, and an elevation/shadow scale. The owner has
confirmed deriving from existing roles rather than extending the surface.

Adding either would require **resetting D-3 and D-4 to open in the same
change**. If you conclude that derivation genuinely cannot work, **stop and
escalate** — do not extend the token surface on your own judgement.

## 5. Change scope

| File | Purpose |
|---|---|
| `crates/snora/src/design/render.rs` (new; name at your discretion) | the design-gated entry point |
| `crates/snora/src/design/render/tests.rs` (new) | per-preset visibility tests |
| `crates/snora/src/design.rs` | declare + re-export |
| `crates/snora/src/overlay/dialog.rs` | a card-rendering path taking style input |
| `crates/snora/src/render.rs` | a dim path taking style input |
| `docs/src/design/` | document the surfaces and the derivation |
| `CHANGELOG.md` | `[Unreleased]` **Added** |

## 6. Explicit non-change scope

Do **not**:

- Add a `Tokens` field to `AppLayout`. `snora-core` has **no dependencies
  at all**; adding `snora-design` would invert the architecture and tax
  engine-only builds. RFC-039 §"The mechanism" explains it.
- Change `snora::render`'s signature or its output. See §8.
- Add or modify any token role or scale (§4).
- Touch `WARNING_COLOR` in `crates/snora/src/toast.rs`. Toasts render on
  the design-**inactive** path; changing them breaks the gating invariant.
  Still deferred.
- Restyle the sheet. It already has `opaque()` and edge-aware rounding; it
  is not broken the way the dialog is.
- Touch `crates/snora-core/` or `crates/snora-design/`.

## 7. Required implementation

### Step 1 — The entry point

```rust,ignore
pub fn render<'a, Message>(
    layout: AppLayout<Element<'a, Message>, Message>,
    tokens: &Tokens,
) -> Element<'a, Message>;
```

Re-exported as `snora::design::render`. It should share the existing
z-stack composition rather than duplicating it — extract the layer assembly
so both entry points call one implementation, with styling passed in.
**Duplicating `render.rs`'s layer logic is the failure mode to avoid**;
the z-stack order is a documented contract and two copies will diverge.

### Step 2 — The dialog card

Fill `surface_raised`, edge `border`, `radius.lg`, padding `spacing.lg`.

A **border-defined card, not a shadow-defined one** — deliberate, per
RFC-039. Shadows are close to meaningless in the high-contrast presets, and
there is no shadow token anyway.

### Step 3 — The dim

This is the hard part and the likeliest source of a defect.

Derive the scrim from existing roles. **You propose the derivation and its
rationale** — it must be a deterministic function of tokens, not a magic
constant, and it must be flagged in the review request as a judgement call,
the way RFC-038's mapping ambiguities were.

Test it against **all four presets**. Specifically consider the two
clamping cases that bit RFC-038:

- `light` — `background` is pure white `rgb(1,1,1)`;
- `high_contrast_dark` — `background` is pure black `rgb(0,0,0)`.

A derivation that shifts "away from" or "toward" a colour already at a
luminance boundary will not move. RFC-038's `shift_away_from` handles this
with a fallback; look at it before writing your own
(`crates/snora-widgets/src/design/theme.rs`).

A scrim that works in `light` and vanishes in `high_contrast_dark` is the
same defect class as RFC-038's invisible borders. Assume it will happen
unless a test proves otherwise.

### Step 4 — Docs

Document both surfaces under `docs/src/design/`, including the derivation
and why the card is border-defined. Note that the two render entry points
exist and when to use each.

## 8. The gating invariant — non-negotiable

With `design` inactive, `snora::render`'s output must be **unchanged**.

The proof is that **`crates/snora/tests/render_semantics.rs` passes
without modification**. If a render-semantics test needs changing, the
invariant is broken — stop and escalate rather than adjusting the test.

That suite encodes the z-stack behavioural contract. It is the thing most
likely to be quietly damaged by refactoring layer assembly in Step 1, and
the thing whose damage is least visible in a diff.

## 9. Required tests

```bash
cargo fmt --check -p snora
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test -p snora --lib --all-features
cargo test -p snora --test render_semantics     # MUST pass unmodified
cargo check -p snora --no-default-features
cargo check -p snora --no-default-features --features widgets
mdbook build docs && mdbook test docs
```

New tests, in `design/render/tests.rs`:

| Test | Assertion |
|---|---|
| Dim visible, all presets | The derived scrim is distinguishable from `background` by a modest contrast floor — pick a defensible number and justify it, as RFC-038 did with 1.5:1 |
| Card distinguishable, all presets | Card fill vs page background clears the same kind of floor |
| Card text contrast | `text_primary` on the card fill meets WCAG AA |

## 10. Acceptance criteria

RFC-039 §Acceptance criteria 1–6. Restated for checkability:

1. `snora::design::render(layout, &tokens)` exists behind `design`.
2. Dialogs through it have a card: fill, border, radius, padding — all
   token-derived.
3. The dim is token-derived and passes the per-preset visibility test.
4. **`render_semantics` passes unmodified.**
5. No change under `crates/snora-core/` or `crates/snora-design/`.
6. RFC-036 covenant compliance stated explicitly in the review request.

## 11. Prohibited shortcuts

- Do not duplicate the z-stack layer assembly.
- Do not modify a render-semantics test to make it pass.
- Do not add a token role to avoid a hard derivation — escalate instead.
- Do not use a hardcoded alpha or colour anywhere in the new path. The
  entire point is that `rgba(0,0,0,0.4)` was a magic constant.

## 12. Compatibility and security

**Compatibility.** Additive, feature-gated. Adopters see a visual change —
a **Changed** entry in the v0.27 migration guide, not **Fixed**. State
gating-invariant compliance explicitly.

**Security.** No new data flow, dependency or integration. Confirm in the
review request.

## 13. Required evidence

- The new entry point and both surface diffs.
- Per-preset test output for dim and card visibility.
- `render_semantics` output, plus `git diff --stat -- crates/snora/tests/`
  showing it is **empty**.
- Both `--no-default-features` check results.
- `git diff --stat -- crates/snora-core crates/snora-design` — empty.
- Your scrim derivation and its rationale, called out for review.

## 14. Required review-request format

Per workflow policy §9.2 and the packaging convention: `README.md` entry
point, full `review-request.md`, `evidence/`, under
`.git-exclude/review-request/039-engine-surface-styling/`. **State the
single entry-point path to hand to the reviewer** in the completion
summary.

**Requested review focus:** the scrim derivation, and whether the shared
z-stack refactor preserved the layer contract.

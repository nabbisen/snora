# Developer Handoff — RFC-061 pointer target size

**Governing RFC.** [RFC-061](../../done/061-pointer-target-size-is-unasserted.md)
**Status.** Inherited from RFC-061 — Accepted (owner, 2026-08-18).
**Release target.** 0.36.0.
**Implementation units.** One.

---

## 1. Task title

Assert the height axis of the 24×24 pointer-target rule in `snora-design`, mark
the width axis review-only with its reason, resolve `chip`'s dismiss control,
and note that no built-in preset role is translucent.

## 2. Purpose

`accessibility-checklist.md` mandates a 24×24 logical-pixel minimum pointer
target, 44×44 preferred. **Neither figure is referenced by any test** — zero
occurrences across `crates/`.

That is RFC-058's shape exactly: a rule written down, never asserted, free to be
violated silently. RFC-058's version had been shipping a WCAG failure for the
role's entire life before an outside evaluator measured it. Raised by tekstide,
who noted the connection themselves.

## 3. The finding that shapes the task — the two axes are not alike

**Do not try to assert both axes.** The reason is structural, not effort:

- **Height is token-derivable.** `line_box + 2 × vertical_padding`, and both
  terms are token values. That is the same property that lets the contrast suite
  run without a renderer, and it makes a height assertion a pure computation
  over `Typography` and `Spacing`.
- **Width is not.** `content_advance + 2 × horizontal_padding`, and
  `content_advance` depends on the string, the font, and the shaping engine.
  snora cannot compute it, and `render_semantics` asserts composition rather
  than geometry.

So the deliverable is **both, split by axis**: assert height, mark width
review-only *and say why*. A reader must be able to tell how much the checkbox
is worth.

Both tempting shortcuts are wrong. Asserting nothing repeats RFC-058. Claiming a
24×24 assertion promises a guarantee on an axis we cannot measure — which is the
worse of the two, because it would look like the problem was solved.

## 4. The candidate violation, computed

`crates/snora-widgets/src/design/chip.rs:175–177`:

```rust,ignore
let remove_btn = button(text("×").size(style::text::label_size(tokens)))
    .padding([tokens.spacing.xs, tokens.spacing.xs])
```

With shipped tokens — `label` role 14.0 / 1.2, `spacing.xs` 4.0:

| axis | computation | result |
|---|---|---|
| height | `14.0 × 1.2 + 2 × 4.0` | **24.8** — clears 24 by 0.8 |
| width | one `×` glyph advance + `2 × 4.0` | **≈ 15** — does not plausibly clear 24 |

The chip's main body (`:143`, `:171`) uses `[xs, sm]` — same 24.8 height, and
wide enough by virtue of its label.

**Measure before fixing.** The width figure is an estimate precisely because
width is not computable at the token level; if you can obtain a real measurement
(a rendered probe outside the test suite, a font-metrics calculation for the
shipped default), do that and report the method. Do not treat ≈15 as established
fact — it is the reason to look, not the finding.

## 5. What to build

### 5.1 The height assertion — `crates/snora-design/src/tests.rs`

A pure test asserting that for every `TextRole` × padding-step combination, the
computed line box plus twice the vertical padding meets 24.

**Q-2 is decided: assert every combination, not only those a prefab uses.**
Enumerating "the ones a control actually uses" means re-deriving a
hand-maintained list of call sites — the exact failure mode that made three
handoff scopes short this cycle. Asserting the full matrix is a stronger ratchet
and costs nothing to compute. Record that reasoning in the test's doc comment.

Expect some combinations to fail the 24 bar legitimately (`xs` padding with a
small role is close to the line). If any combination a prefab *actually uses*
fails, that is a defect — report it before repairing, exactly as RFC-058
required.

Follow the module's existing style; this sits alongside the contrast pairs.

### 5.2 The width axis — `accessibility-checklist.md`

State that the width axis is **review-only**, and *why*: content advance is not
token-derivable, so it cannot be asserted without a renderer. Distinguish it
from the height axis, which now is asserted.

The point of the sentence is that a future reader knows which half is enforced.
An unqualified "target size is checked" would be worse than today's silence.

### 5.3 `chip`'s dismiss control — Q-1

Three candidates, in the RFC. Choose and justify:

- **Increase horizontal padding** to `sm` — cheapest, small appearance change.
- **A minimum width** via iced layout — more precise, but snora then owns that
  geometry.
- **Exempt it.** WCAG 2.5.8 exempts controls whose function is available
  elsewhere. snora's chip offers no other dismissal route, so this is probably
  **not** available — verify before relying on it.

**This is an appearance change to a shipped primitive.** If you change it, it
needs a CHANGELOG entry and a migration-guide note, the same treatment RFC-058's
border change received.

**Its blast radius is probably zero, and that is not a reason to be casual.**
Consumer replies (2026-08-18) confirm orbok is the **only** prefab-widget
consumer — apimokka has zero `snora::widget::*` call sites and arama has removed
`snora-widgets` from its build graph entirely. orbok's own description of its UI
names a side bar, a tab bar, result cards and source cards, and **does not
mention chips**. Absence of mention is not confirmation, so treat chip usage as
**unknown**, ship the migration note regardless, and do not use "nobody uses it"
as licence to skip the failing-first evidence.

If the measurement in §4 turns out to clear 24 after all, say so and change
nothing — a fix applied to a non-defect is worse than no fix.

### 5.4 The 44×44 preferred bar

Record its status per role/padding combination rather than leaving a second
unasserted number beside the first. It is *preferred*, not mandatory, so this is
a statement of where we stand, not a second assertion.

### 5.5 Q-3 — the `composite_over` note (folded in)

`accessibility-checklist.md:70` reads:

> `[ ]` If the primitive uses an alpha/translucent color, it is composited over
> the tested background before the contrast ratio is computed.

**Keep the rule.** Add a note that **no built-in preset role is translucent** —
verified, every colour across all four presets is `Color::rgb(...)`, not one
`rgba` — so the rule applies to applications introducing their own translucent
tokens rather than to snora's own primitives.

Re-run the check (`grep -rn "rgba" crates/snora-design/src/presets/`) to confirm
it still holds before writing the note.

**Do not weaken or remove the rule.** It becomes live the moment a translucent
token is added, and is worth keeping armed.

### 5.6 Q-3's margin — the thin one

Record that the `label` role at `xs` padding clears 24 by **0.8 logical
pixels**, as a fact the next token edit must respect — the same treatment
RFC-058 gave `dark`'s 4.526:1 margin. A margin that thin is a trap for a future
spacing change, and recording it is what makes the trap visible.

## 6. Change scope

| File | Purpose |
|---|---|
| `crates/snora-design/src/tests.rs` | the height assertion (§5.1) |
| `docs/src/contributing/accessibility-checklist.md` | width review-only, 44×44 status, Q-3 note, the thin margin (§5.2, §5.4, §5.5, §5.6) |
| `crates/snora-widgets/src/design/chip.rs` | only if Q-1 concludes a fix (§5.3) |
| `docs/src/guides/migration-0.35-to-0.36.md` | only if `chip` changes |
| `CHANGELOG.md` | **Added** for the assertion; **Fixed** if `chip` changes |

## 7. Explicit non-change scope

Do **not**:

- **Assert the width axis**, or add renderer-based geometry testing.
- **Add a `Spacing` or `Typography` field.** Both are under RFC-036's
  additive-only covenant; needing a new token is a separate decision with a gate
  cost — stop and report.
- **Change spacing or typography *values*** unless §5.1 finds a real defect —
  and if it does, RFC-036's accessibility carve-out applies with its
  failing-first order mandatory.
- **Audit application-supplied controls.** snora asserts its own primitives.
- **Weaken the `composite_over` rule** (§5.5).
- Modify `render_semantics.rs`.

## 8. Required tests

```bash
cargo test -p snora-design
cargo test -p snora --lib --all-features
cargo test -p snora --test render_semantics     # MUST pass unmodified
cargo test -p snora-widgets --features design   # if chip changes
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --all --check
mdbook build docs && mdbook test docs
```

If §5.3 changes `chip`, failing-first evidence applies: show the height/width
measurement before, the change, and the measurement after.

## 9. Required evidence

- The height assertion and its full role × padding matrix output.
- The `chip` measurement (§4), **with the method stated** — this is the number
  the RFC could not establish.
- Q-1's decision and its justification; the diff if `chip` changed.
- The checklist diff showing all four documentation items (§5.2, §5.4, §5.5,
  §5.6).
- The re-run `rgba` grep.
- `render_semantics` output and `git diff --stat -- crates/snora/tests/` empty.
- CHANGELOG, and the migration note if `chip` changed.

## 10. Acceptance criteria

RFC-061 §Acceptance criteria 1–7. The two most likely to go wrong:

- **2** — the checklist must make the asserted/review-only split *legible*. The
  failure mode is a reader concluding target size is enforced.
- **3** — `chip` measured before any change, with the method reported.

## 11. Required review-request format

Per workflow policy §9.2: `README.md` entry point, full `review-request.md`,
`evidence/`, under
`.git-exclude/review-request/061-pointer-target-size-is-unasserted/`.
**State the single entry-point path** in the completion summary.

**Requested review focus:** the `chip` measurement and its method, and whether
the checklist now makes it impossible to read the width axis as enforced.

# RFC 061 — Pointer target size is a checklist rule with no assertion, and one control looks under it

**Status.** Accepted (owner, 2026-08-18). Handoff:
[`handoffs/061-…`](../handoffs/061-pointer-target-size-is-unasserted/implementation-handoff.md)
**Tracks.** Accessibility. Raised by **tekstide** (Q4 and Q3, 2026-08-17);
answered in correspondence 2026-08-18 and not yet reflected in the repository.
**Touches.** `crates/snora-design/src/tests.rs`,
`crates/snora-widgets/src/design/chip.rs` (possibly),
`docs/src/contributing/accessibility-checklist.md`, `CHANGELOG.md`.
**Release target.** 0.36.0.

## Summary

`accessibility-checklist.md` mandates a 24×24 logical-pixel minimum pointer
target, 44×44 preferred. **Neither figure is referenced by any test** — zero
occurrences across `crates/`. It is a review item wearing the appearance of an
enforced one.

This is the same shape as RFC-058: a rule written down, never asserted, and
therefore free to be violated silently. RFC-058's version of it had been
shipping a WCAG failure for the role's entire life before an outside evaluator
measured it.

**And there is a candidate violation already.** `chip`'s dismiss control
computes to roughly **15 × 24.8** logical pixels — clearing the height bar by
0.8 and, on the width axis, very unlikely to clear it at all.

## The evidence

`crates/snora-widgets/src/design/chip.rs:175–177`:

```rust,ignore
let remove_btn = button(text("×").size(style::text::label_size(tokens)))
    .padding([tokens.spacing.xs, tokens.spacing.xs])
```

With the shipped tokens — `label` role at 14.0 / 1.2 line height, `spacing.xs`
at 4.0:

| axis | computation | result |
|---|---|---|
| height | `14.0 × 1.2` + `2 × 4.0` | **24.8** — clears 24 by 0.8 |
| width | one `×` glyph advance + `2 × 4.0` | **≈ 15** — does not plausibly clear 24 |

The chip's main body (`:143`, `:171`) uses `[xs, sm]`, so it shares the same
24.8 height and is wide enough by virtue of its label.

## The finding that shapes this RFC: the two axes are not alike

**Height is token-derivable. Width is not.**

A control's height is `line_box + 2 × vertical_padding`, and both terms are
token values — exactly the property that makes the contrast suite possible
without a renderer. A height assertion is a pure computation over `Typography`
and `Spacing`, testable in `snora-design` alongside the twelve contrast pairs.

Width is `content_advance + 2 × horizontal_padding`, and `content_advance`
depends on the string, the font, and the shaping engine. snora cannot compute it
without a renderer, and `render_semantics` asserts composition rather than
geometry.

So the honest position is neither "assert target size" nor "mark it review-only"
but **both, split by axis** — and saying which is which, so a reader knows
exactly how much the checkbox is worth.

This is worth stating because the tempting alternatives are both wrong:
asserting nothing repeats RFC-058, and claiming a full 24×24 assertion would
promise a guarantee on an axis we cannot measure.

## Scope

1. **Assert the height axis** in `snora-design`'s test module: for every
   `TextRole` and every padding step a prefab control uses, the computed line
   box plus twice the vertical padding meets 24. Pure, no renderer.
2. **Mark the width axis review-only, explicitly**, in the checklist — stating
   that it is not enforceable at the token level and why, so it is not mistaken
   for an assertion later.
3. **Resolve `chip`'s dismiss control.** See Q-1.
4. **Record the 44×44 preferred bar's status** — it is met by some
   role/padding combinations and not others; say which, rather than leaving a
   second unasserted number beside the first.
5. **Answer tekstide's Q3 in the same file** — see below.

## Folded in: `composite_over` is guidance nothing we ship exercises (Q3)

`accessibility-checklist.md:70` reads:

> `[ ]` If the primitive uses an alpha/translucent color, it is composited over
> the tested background before the contrast ratio is computed.

That guidance is **correct and stays**. What is missing is that **no built-in
preset role is translucent** — verified: every colour across all four presets is
`Color::rgb(...)`, and there is not one `rgba`. So the compositing path is not
exercised by anything snora ships.

tekstide asked precisely this, because they have a modal scrim at alpha 0.55 and
had to reason it through, and said `composite_over` was *"the piece we would
most likely have got wrong on our own."* They deserved to know it is correct but
not battle-tested by us.

This is folded in rather than given its own RFC because it is one caveat in the
same file this RFC already opens — and because leaving it in outbound
correspondence only would be the same defect RFC-059 exists to stop: an answer
that lives where the consumer cannot find it.

**Scope:** one note attached to that checklist line stating that no built-in
role is translucent today, so the rule applies to applications introducing their
own translucent tokens rather than to snora's own primitives. Do not weaken or
remove the rule — a rule that becomes live the moment a translucent token is
added is worth keeping armed.

## Non-goals

- **No renderer-based geometry testing.** `render_semantics` asserts
  composition, not pixels, and this RFC does not change that.
- **No new `Spacing` or `Typography` fields.** Both are under RFC-036's
  additive-only covenant; if the fix needs a new token, that is a separate
  decision with a gate cost.
- **No change to the spacing or typography *values*** unless Q-1 concludes a
  defect, in which case RFC-036's accessibility carve-out applies and its
  failing-first order is mandatory — as in RFC-058.
- **No audit of application-supplied controls.** snora asserts its own
  primitives; what an application puts in `body` is its own.

## Open questions

**Q-1 — how should `chip`'s dismiss control be fixed?** Three candidates, and
the choice is a design decision rather than an implementation detail:

- **Increase its padding** to `sm` horizontally — cheapest, changes appearance
  slightly, keeps the glyph.
- **Give it a minimum width** via iced's layout rather than padding — more
  precise, but it is geometry snora would then own.
- **Conclude it is exempt.** WCAG 2.5.8's exceptions include controls whose
  function is available elsewhere. A chip's dismiss may qualify *if* the chip
  itself is removable another way — which for snora's chip it is not, so this
  is probably not available.

Measure before choosing, and note that it is an appearance change to a shipped
primitive with one known consumer (orbok).

**Q-2 — should the height assertion cover every role × padding combination, or
only those a prefab control actually uses?** The former is a stronger ratchet
and will assert combinations nothing renders; the latter is precise but
re-derives a hand-maintained list of call sites — the failure mode that made
three handoff scopes short this cycle. Suggest the former, with the reasoning
recorded.

**Q-3 — is 24 the right floor given the `xs` step is 4.0?** At `xs` padding, the
`label` role clears 24 by 0.8 logical pixels. Any future reduction in either
token breaks it. That margin is thin enough to be worth stating in the same way
RFC-058 recorded `dark`'s 4.526:1 — as a fact the next token edit must respect.

## Acceptance criteria

1. A pure height-axis assertion exists in `snora-design` and passes.
2. The checklist states which axis is asserted and which is review-only, and
   why width cannot be asserted at the token level.
3. Q-1 answered: `chip`'s dismiss control measured, and either fixed with
   failing-first evidence or documented as exempt with the reasoning.
4. The 44×44 preferred bar's status recorded per role/padding combination.
5. Q-3's margin recorded.
6. **tekstide's Q3 answered in the checklist**: the `composite_over` line
   carries a note that no built-in preset role is translucent, with the rule
   itself unchanged.
7. `render_semantics` passes unmodified.

## Compatibility and security

**Compatibility.** The assertion is additive. If Q-1 changes `chip`'s padding,
that is a rendered appearance change on the `design` path affecting the one
consumer using prefab widgets, and needs a migration-guide note — the same
treatment RFC-058's border change received.

**Security.** None.

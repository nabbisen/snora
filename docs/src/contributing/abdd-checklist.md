# ABDD compliance checklist

**ABDD — Accessible By Default and by Design** — is Snora's first
principle. Every layout surface uses logical edges (`Edge::Start`,
`Edge::End`, `LayoutDirection`) rather than physical left/right so that
the same code works for LTR and RTL readers.

This checklist makes ABDD correctness a repeatable review gate rather
than something that depends on reviewer memory. Complete it for any
change that touches positioning, alignment, mirroring, or anchoring.

## Scope

```text
[ ] Does this change position, align, order, mirror, anchor, or label a UI surface?
[ ] If no, state why ABDD does not apply and skip the rest.
```

## Logical direction

```text
[ ] Uses Start/End or LayoutDirection instead of hardcoded Left/Right
    where the concept is logical (sidebar side, sheet edge, toast anchor).
[ ] Any physical Left/Right use is intentionally physical and documented
    as such in a comment ("intentionally physical: numbers always LTR").
[ ] Start/End resolves consistently and correctly under both
    LayoutDirection::Ltr and LayoutDirection::Rtl.
```

## Public API

```text
[ ] New public names avoid LTR-only assumptions in their identifiers.
[ ] New enum variants use logical terms (Start/End) where appropriate,
    not Left/Right.
[ ] Default values are sensible under both LTR and RTL.
```

## Examples and docs

```text
[ ] At least one example or doc snippet demonstrates RTL behavior
    if the changed surface is direction-sensitive.
[ ] Docs state that Snora handles layout direction, not full
    translation/localization (ABDD is a layout discipline).
```

## Tests

```text
[ ] Pure-logic tests (unit or doctest) cover both LTR and RTL where feasible.
[ ] Engine render-semantics tests cover mirroring if the behavior lives
    in the render engine (e.g. sheet corner, toast anchor side).
```

## Accessibility wording

```text
[ ] The change does not overclaim full accessibility or full i18n.
[ ] Tooltip or label text is required (or documented as required) for
    any visual-only control introduced.
```

## Source-comment convention

For any helper that resolves logical edges to physical sides, the
comment must make the translation explicit:

```rust,ignore
// Resolve logical Start/End to the physical side required by iced layout.
```

Comments that say "left sidebar" or "right panel" without qualification
violate ABDD unless the side is intentionally physical.

## What ABDD is not

ABDD covers **layout direction** only. Snora does not own:

- bidirectional text shaping (iced's text layer);
- locale-driven number or date formatting (`icu`, `num-format` crates);
- full screen-reader or keyboard-navigation semantics (see
  [overlay interaction semantics](../reference/overlay-interaction-semantics.md)
  Laws 7 and 8);
- icon mirroring for RTL — use direction-aware Lucide icon variants.

The getting-started [direction guide](../guides/direction.md) explains
the full ABDD picture for application developers.

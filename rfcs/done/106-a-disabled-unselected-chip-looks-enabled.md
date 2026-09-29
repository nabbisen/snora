# RFC-106 — A disabled unselected chip looks enabled

**Status.** Done — shipped v0.52.0. Q-1 = 0.45, Q-2 = yes, Q-3 = 0.52.0 before the cut.
**Raised.** 2026-09-29, architect. Found by RFC-103's state register, which
records it as `Cue::Missing` and **pins** the gap, so a test fails when it is
closed.
**Release target.** 0.52.0, before the cut (Q-3). **Priority.** Small.

---

## The finding

A disabled **unselected** chip is indistinguishable from an enabled one:

| | light | dark | hc_light | hc_dark |
|---|---|---|---|---|
| unselected, enabled vs disabled (fill) | 1.04 | 1.05 | 1.00 | 1.00 |

`chip_style_unselected`'s disabled arm only halves the fill's alpha, and the fill
is `surface`, on a page that is nearly `surface`. **The label colour and the
border do not change at all.** The selected chip escapes this only because its
fill is `accent` (2.64–3.59:1 enabled vs disabled).

A user sees a control that looks available and does nothing. This is also
tekstide's REQ-004 for a state that matters (RFC-103 Q-2 ruled disabled one).

## Why before the cut

- **0.52.0 already invalidates visual baselines** (RFC-104's label sizes).
  Shipping this now means consumers re-baseline once rather than twice.
- **0.52.0 ships RFC-103's state register.** Without this RFC it ships with one
  known `Missing` cue; with it, the register is complete for every state snora
  draws.
- **Small, and outside the frozen surface.** `chip` is a design primitive, which
  RFC-036 excludes from the covenant. `snora-style` is not changed.

## Proposal

**R-1 — the label carries the disabled cue.** The fill cannot, since `surface` is
too close to the page, so dim the label and the border in the disabled arm.
Measured, over the half-alpha disabled fill:

| alpha | label enabled vs disabled | disabled label vs its fill | border enabled vs disabled |
|---|---|---|---|
| **0.45** | **3.18 – 6.03** | 2.20 – 3.74 | 2.09 – 6.27 |
| 0.5 | 2.82 – 5.21 | 2.44 – 4.43 | 1.96 – 5.28 |

(Ranges across light / dark / hc_light / hc_dark. The low end is dark for
enabled-vs-disabled, and light for the label against its fill.)

**The disabled label's own contrast (2.20–3.74:1) is below 4.5, and that is
permitted:** WCAG 1.4.3 exempts inactive user-interface components. The RFC
says so explicitly, so no reader mistakes it for a regression.

**R-2 — flip the register's pin.** `ChipUnselectedDisabled` becomes a
`Luminance` cue on the **label** channel, asserted at the measured floor. That
follows RFC-103's per-channel naming for transparent-fill controls. The pinned
test fails, as designed, and is replaced in the same change.

## Open questions

**Q-1 — which alpha?** **Suggest 0.45**, the factor `snora_style` already uses
for disabled buttons (`disabled_alpha`). It is the only one of the two that
clears 3.0:1 enabled-vs-disabled in all four presets, and it makes disabled look
the same across snora's primitives. 0.5 keeps the chip's existing number but
reaches only 2.82 on dark.

**Q-2 — does the selected chip change too?** Its fill already carries a cue
(≥ 2.6). **Suggest yes: dim its label and border by the same factor**, so
"disabled" is one visual treatment on both states rather than two. The handoff
re-measures the selected-disabled floor, which can only rise.

**Q-3 — release.** **Suggest 0.52.0, before the cut**, for the reasons above.

## Acceptance criteria

1. The unselected disabled chip's label enabled-vs-disabled is asserted at the
   measured floor (≥ 3.0 at the suggested alpha), **shown failing** with the dim
   removed.
2. The register's `Missing` entry is replaced, the pin's failure is shown, and
   it is removed in the same change.
3. The selected chip as ruled in Q-2, with its floor re-measured.
4. `snora-design` and `snora-style` unchanged (quote the diff). CHANGELOG under
   Fixed, naming it as found by RFC-103's register.

---

## Rulings, 2026-09-29

**Q-1:** alpha **0.45**. **Q-2:** the selected chip's label and border are dimmed
by the same factor. **Q-3:** 0.52.0, before the cut.

**Found before the handoff:** `snora_style::button`'s `disabled_alpha` (×0.45)
is **private** to a frozen crate. The chip reproduces the factor as a named
constant, with a comment pointing at `disabled_alpha`, rather than widening
`snora-style`'s public surface for one value.

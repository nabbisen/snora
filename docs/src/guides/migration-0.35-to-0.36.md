# Migration 0.35 → 0.36

> `chip::removable`'s dismiss ("×") button gained a fixed minimum width
> — a WCAG 2.5.8 pointer-target-size repair, not a restyle.

## Who is affected

Every application using `snora_widgets::design::chip::removable` (or
`snora::design::chip::removable`, reached through the `snora` facade
with `widgets` + `design` enabled). `chip::filter` is unaffected —
only the separate dismiss button on a removable chip changed.

## What changed

The dismiss button's rendered width. Previously it sized to its content
(`text("×")` plus `2 × spacing.xs` padding) with no explicit width —
measured at **15.0 logical pixels** total against iced's shipped
fallback font (FiraSans-Regular), against the WCAG 2.5.8 mandatory
24×24 pointer-target floor. It now has an explicit minimum width,
computed as `line_box + 2 × spacing.xs` (the same formula its height
already resolved to), making it square: **24.8 logical pixels** on a
side with the shipped `label` role and `spacing.xs` token values.

No field, type, or function signature changed. `chip::removable`'s own
signature is identical.

## Why it changed

`accessibility-checklist.md` has mandated a 24×24 logical-pixel minimum
pointer target since it was written, but nothing asserted it —
RFC-058's shape exactly: a rule written down, never enforced, free to
be violated silently. Raised by tekstide, who noted the parallel
themselves. The dismiss button's height cleared the floor (24.8px,
computed from token values, now asserted by
`cargo test -p snora-design`), but its width — dependent on the
rendered glyph's advance, the font, and the shaping engine, none of
which snora can compute without a renderer — did not, measured
directly against the actual shipped fallback font rather than
estimated.

Padding alone could not fix this reliably: even increasing padding to
`spacing.sm` reaches only 23.0px against that same font, one pixel
short. The fix instead forces an explicit minimum width on the
button's content, computed from the same tokens that already determine
its height, so the control is square and the fix tracks any future
token change automatically.

## Mechanical migration

None. No API changed; `chip::removable`'s call sites are unaffected.

## Behavioral migration

**This is an appearance change**, not a behavior change: the dismiss
button on a removable chip is visibly wider — from content-hugging
(~15px) to a fixed square (~24.8px at the shipped default tokens).
Re-check any screenshot tests or visual regression baselines that
include `chip::removable`.

The direction is one-way: the pointer target grows. Nothing that was
clickable becomes harder to reach; some space around the "×" glyph
that was previously outside the clickable area is now inside it.

## Deprecated aliases and removal schedule

None — this is a rendering change, not an API change.

## Examples before/after

No repository example constructs a `chip::removable` with hand-computed
dimensions; all examples call it through its normal signature and pick
up the new width automatically. No example changes were required.

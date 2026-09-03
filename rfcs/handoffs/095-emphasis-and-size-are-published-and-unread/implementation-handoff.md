# Developer Handoff — RFC-095 `Emphasis` and `Size`

**Governing RFC.** **RFC-095** — find it under `rfcs/accepted/`, `rfcs/done/` or `rfcs/archive/` via the index at [`rfcs/README.md`](../../README.md). *(Not linked by folder: an RFC moves between folders as it ships.)*
**Status.** Accepted (owner, 2026-09-03).
**Release target.** **0.45.0.** The decision is taken at 0.44.0.
**Blocked on Q-1, and Q-1 is blocked on the architect.** Do not start.

---

## Why there is nothing for you to do yet

**Q-1 — consumers or removal — cannot be ruled from inside this repository.**

The default answer is removal: 24 minors, two public enums, nothing reads either.
But **absence of demand is not evidence here.** Nobody asks for an enum they did
not know was inert, and this project has made that exact error already: RFC-078
counted apimokka's decline as evidence against focus trapping, when the decline
was an echo of a constraint we had published (`design-decisions.md`, corrected
2026-09-02).

So the teams get asked once, in the next letter that has its own reason to exist
— which now exists, because RFC-093's Q-1 obligation is waiting for the same
letter. **That letter is the architect's.** When answers come back, Q-1 is ruled
and this handoff gains a Unit 1.

Nothing here is yours until then. This document exists so the dependency is
written down rather than remembered.

## Rulings that do not wait

**Q-2 — if `Size` survives Q-1, it is renamed in the same break.** Its problem is
not only that nothing reads it: it shadows `iced::Size`, which the engine uses
heavily (`responsive.rs`, `design/render.rs`). A consumer writing
`use snora::design::Size` gets an inert enum and no error. Renaming is exactly as
breaking as removing, so if we are paying the covenant's price at all, both are
paid at once or neither is.

**Q-3 — the "what else is unread?" sweep is not in this RFC.** It is RFC-094's
shape applied to the API surface instead of the gate register, and bundling them
would sink both. Open it separately if RFC-094's sweep goes well.

## What removal will cost, so it is not a surprise later

`Emphasis` and `Size` are named in RFC-036's frozen surface (RFC-036:111).
Removal is a **forbidden change** under the additive-only covenant, permitted
only by its own reopening condition:

> the implementing RFC must say so explicitly and **reset D-3 and D-4 to open**
> in the same change. It must not proceed and rationalise afterwards.

So if Q-1 rules removal, the implementing change **must** reset D-3 (token model
stable ≥2 consecutive minors) and D-4 (style bridge stable ≥2 consecutive
minors) to ⬜ in the same commit — not afterwards, not in a follow-up. They are
re-earned by 0.47.0 at the earliest.

**Do not treat that as paperwork.** The covenant's stated value is *"entirely in
its being expensive to reverse"*. A reopening that skips the reset is a covenant
that has quietly stopped existing.

## Acceptance criteria

Deferred with Q-1. They will name whichever of these applies:

- **If removal:** both enums gone, the facade re-export updated, D-3 and D-4
  reset to ⬜ **in the same change**, the covenant exception stated in the
  CHANGELOG rather than implied, and a migration guide entry — this one is
  genuinely breaking, unlike 0.43.0.
- **If retention:** the consumer that justifies each exists, or
  `variants.rs`'s module doc says plainly that they are reserved vocabulary and
  why that is worth a public name. `Size` is renamed either way.

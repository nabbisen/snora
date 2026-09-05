# Developer Handoff — RFC-095 `Emphasis` and `Size`

**Governing RFC.** **RFC-095** — find it under `rfcs/accepted/`, `rfcs/done/` or `rfcs/archive/` via the index at [`rfcs/README.md`](../../README.md). *(Not linked by folder: an RFC moves between folders as it ships.)*
**Status.** Accepted (owner, 2026-09-03).
**Release target.** **0.45.0.** The decision is taken at 0.44.0.
**Blocked on Q-1, and Q-1 is blocked on the architect.** Do not start.

---

## Q-1 is answered: remove. You may start.

**All six adopting teams replied and all six reference neither enum.** Three
answered by enumerating what they *do* use rather than grepping the two names —
the stronger method, and not one we asked for. Nobody wants them wired up.

Full answer and method in RFC-095's own "Q-1's answer" section; the review record
is `.git-exclude/reviewed/` alongside the reply letters.

**One caution carried over from how the question was asked.** It was put to them
because *absence of demand is not evidence when nobody knew the enums were
inert* — the RFC-078/apimokka error. Six teams checking their trees and reporting
zero **is** evidence. The distinction matters if anyone reopens this later: this
removal rests on six positive checks, not on silence.

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

1. **Both enums removed** from `crates/snora-design/src/variants.rs`, and the
   re-exports updated in `snora-design/src/lib.rs` and `crates/snora/src/design.rs`.
   `Tone` and `Density` stay — both are read.
2. **`variants.rs`'s module doc updated.** Its table currently says which enums
   are read by what; after this it describes two, not four.
3. **D-3 and D-4 reset to ⬜ in the same change**, in
   `docs/src/contributing/api-freeze-review.md`. Not afterwards, not in a
   follow-up. RFC-036's reopening condition is explicit that the covenant's value
   is *"entirely in its being expensive to reverse"*, and a reopening that skips
   the reset is a covenant that has quietly stopped existing. **State in the same
   commit message that this is a forbidden change taken deliberately.**
4. **CHANGELOG entry naming the covenant exception**, not implying it.
5. **A migration guide entry** — this one is genuinely breaking, unlike 0.43.0
   and 0.44.0. It should say what we know: six teams checked and none referenced
   either, so the expected impact is nil, and `cargo build` will name the type if
   we were wrong about a seventh.
6. Do not touch `Tone`, `Density`, or anything else in the frozen surface.

**If removal turns out to be harder than deleting two enums** — if something in
the workspace or an example does reference them after all — stop and report it.
That would mean our own check was wrong, which is worth more than the removal.

# RFC 095 — `Emphasis` and `Size` are published vocabulary nothing reads

**Status.** Proposed (2026-09-03).
**Tracks.** Public API / 1.0 readiness. **Severity: Medium.**
**Found by** RFC-093's Q-3, which asked whether the channel register reaches
`Emphasis` and got a more interesting answer than expected.
**Touches.** `crates/snora-design/src/variants.rs`, `crates/snora/src/design.rs`,
`docs/src/contributing/api-freeze-review.md`.
**Release target.** **0.45.0**, with the decision taken at 0.44.0 — see "What
removal actually costs". *(First drafted as "1.0, not sooner" on the belief that
the covenant forbade removal in a minor. It does not; it prices it. Corrected
2026-09-03 after the owner asked why not 0.45.0.)*

## The finding

`snora_design::variants` defines four shared enums. **Two of them are read by
nothing.**

| Enum | Read by |
|---|---|
| `Tone` | `snora_widgets::design::notice`, and `progress` via `snora_style::progress::toned` |
| `Density` | a field on `Tokens`; set by the presets |
| **`Emphasis`** | **nothing** |
| **`Size`** | **nothing** |

Checked 2026-09-02: the only non-test, non-definition reference to either is the
re-export line in `crates/snora/src/design.rs`. No widget and no style function
varies anything by them.

They shipped in **v0.19** (RFC-020…RFC-030) and have been inert for **24 minors**.

## Why this is more than tidiness

**`Size` collides with `iced::Size`.** The engine uses `iced::Size` heavily —
`responsive.rs`, `design/render.rs`. A consumer who writes `use snora::design::Size`
expecting a sizing type gets a variant enum that nothing reads, and no compiler
error tells them so. The name is the worst part of this finding, not the disuse.

**The module doc asserted the opposite.** `variants.rs` said these enums are
*"reused across buttons, chips, notices, and progress"*. Buttons and chips take
neither `Tone` nor `Emphasis` — they take nothing from this module at all.
Corrected 2026-09-02; the sentence had been wrong since v0.19.

**A published type implies a promise.** `snora::design::Emphasis` reasonably
reads as *"some prefab honours emphasis"*. Nothing does. That is the same class
of defect as the withdrawn 1.4.1 claim: not broken code, but a statement about
the code that a consumer could act on and be wrong.

## What removal actually costs

**RFC-036 names both enums in the frozen surface** (RFC-036:111, alongside
`Density`; `variants.rs` is in the frozen file list at :77). Removing either is a
forbidden change.

**Forbidden is not prohibited — it is priced.** The covenant's own reopening
condition:

> If work requires any forbidden change, the implementing RFC must say so
> explicitly and **reset D-3 and D-4 to open** in the same change. It must not
> proceed and rationalise afterwards. The gate's value is entirely in its being
> expensive to reverse.

So the price of removal is exact and payable: **D-3** (token model stable ≥2
consecutive minors) and **D-4** (style bridge stable ≥2 consecutive minors) go
back to ⬜, and are re-earned over the two minors that follow — 0.47.0 at the
earliest.

**That price is smaller than it looks, and paying it now is cheaper than later:**

- The design track already carries D-1 and D-2 open, coupled to gate 1's iced
  major upgrade. Resetting D-3/D-4 blocks nothing that is not already blocked.
- snora is pre-1.0, and 0.41.0 and 0.42.0 both shipped breaking changes in
  minors. Consumers are upgrading through breaks already.
- **After 1.0 this needs a 2.0.** Carrying a public name that shadows
  `iced::Size` past the 1.0 line converts a cheap correction into an expensive
  one, permanently.

**The real question is not mechanical.** It is whether spending the covenant's
reopening on two inert enums cheapens it. The argument that it does not: the
covenant exists to stop churn in a surface under active design, and nothing is
under design here — nothing reads either enum, so no consumer's code changes and
no appearance moves. The argument that it does: covenants die by exception, and
"this one is harmless" is how every exception introduces itself.

**The owner's call, and this RFC asks for it explicitly rather than assuming
either way.**

## Non-goals

- **Not proposing to remove them now.** The covenant forbids it and this RFC
  does not seek an exception.
- **Not proposing to wire them up as a hedge.** Adding `Emphasis` to buttons to
  justify its existence would be building a feature to retire a finding, which is
  backwards. If emphasis variants are worth having, they are worth having on
  their own argument.
- **Not touching `Tone` or `Density`.** Both are read and both are fine.

## Open questions

**Q-1 — consumers, or removal at 1.0?** The honest default is removal: 24 minors
of no demand, from a vocabulary crate whose own doc says these must "stay small".
But **absence of demand is not evidence here** — nobody asks for an enum they
did not know was inert, which is the exact reasoning error RFC-078 made with
apimokka's decline. **Suggest: ask the adopting teams once**, in the next letter
that has its own reason to exist, and let the answer decide. **That letter now
exists**: RFC-093's Q-1 obligation is waiting for the same one — the colour-alone
paragraph — so a single letter carries both questions and neither costs a relay
of its own.

**Q-2 — is `Size`'s name a problem independent of its use?** If it gains
consumers, it still shadows `iced::Size` in every consumer's import list.
Renaming is as breaking as removing. **Suggest: if it survives Q-1, it is
renamed in the same break.**

**Q-3 — how many other published items are unread?** This was found by asking
about one enum. The same question has not been asked of the facade's other
re-exports. **Suggest: not in this RFC** — it is RFC-094's shape, applied to the
API surface instead of the gate register, and bundling them would sink both.

## Acceptance criteria

1. Q-1 answered, with the adopting teams asked rather than inferred from.
2. Whichever way it goes, `variants.rs`'s module doc and the freeze review agree
   with the decision.
3. If removal: recorded against the 1.0 break with the covenant exception stated,
   not smuggled into a minor.
4. If retention: the consumer that justifies it exists, or the RFC says plainly
   that it is kept as reserved vocabulary and why that is worth a public name.

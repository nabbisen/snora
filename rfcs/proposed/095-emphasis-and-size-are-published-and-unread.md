# RFC 095 — `Emphasis` and `Size` are published vocabulary nothing reads

**Status.** Proposed (2026-09-03).
**Tracks.** Public API / 1.0 readiness. **Severity: Medium.**
**Found by** RFC-093's Q-3, which asked whether the channel register reaches
`Emphasis` and got a more interesting answer than expected.
**Touches.** `crates/snora-design/src/variants.rs`, `crates/snora/src/design.rs`,
`docs/src/contributing/api-freeze-review.md`.
**Release target.** **1.0**, or the release that decides it. Not 0.44.0 — see
"Why this cannot just be fixed".

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

## Why this cannot just be fixed

**RFC-036's additive-only covenant freezes the design surface.** Both enums are
public, re-exported through the facade, and inside the freeze. Removing either is
a breaking change and cannot ship in a minor.

So the options are genuinely: **give them consumers, or remove them in the 1.0
break.** There is no third path that ships sooner, which is why this RFC targets
the decision rather than a release.

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
that has its own reason to exist, and let the answer decide.

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

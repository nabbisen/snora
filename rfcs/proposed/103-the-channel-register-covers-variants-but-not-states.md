# RFC-103 — The channel register covers variants, not states

**Status.** Proposed.
**Raised.** 2026-09-26, architect. Source: REQ-004 in
`contributing/consumer-requirements.md` (tekstide: *"every state that matters
also carries a word or shape"*).
**Release target.** 0.52.0. **Priority.** First of three: small, and it corrects
a published overstatement.

---

## The finding

**RFC-093's channel register asserts that snora's semantic *variants* are
colour-only**: toast intent, notice tone, progress tone. The caller supplies the
words, and the division of labour is documented. **It has no notion of
*states*,** which snora draws itself and which the caller's words do not
distinguish:

| State snora draws | Cue today | Asserted? |
|---|---|---|
| Active tab | **Shape** — 2 px underline (RFC-102); indicator ≥ 3.0:1 | Colour pair yes; shape by rendered bounds |
| Sidebar active item | **Fill** vs no fill, ≥ 3.0:1 against the rail (RFC-085) | Colour pair only |
| **Chip selected** | **Colour only** — fill, border and text colour change; same pill. Luminance between states measured **6.19–11.75:1** | **No** |
| Menu open | The dropdown appears (a shape) | Rendered |
| Breadcrumb leaf ("you are here") | **Position only.** At rest a leaf and its ancestors are the same colour; the leaf is plain text, ancestors are buttons with no resting background | No |
| Hover, pressed, disabled | Colour / alpha | — (transient or not a "state that matters"; ruled in Q-2) |

**The chip is the gap that matters.** A selected filter chip carries the same
label text in both states, so unlike a toast, the caller's words do not tell
the states apart unless the caller changes the label, as orbok does with ✓.
**That practice is not documented.**

### A published claim this overstates

RFC-045's position text, in `semantic-accessibility.md`, says ABDD means
*"layout-direction correctness and visual accessibility — contrast, logical
edges, **non-colour status encoding**"*. Since RFC-093 ruled that snora
contributes colour and the caller supplies words for variants, and the chip
state has no non-colour cue at all, **the phrase claims more than snora
provides.** orbok's record cites us for 1.4.1-adjacent material.

## Proposal

**R-1 — a state register, exhaustive and asserted.** In the same shape as
RFC-093's: an enum of every snora-drawn state, each declaring its cue:

- `Shape`, asserted by rendered bounds;
- `Luminance(min)`, asserted by colour pairs between the two states;
- `CallerText`, where the division of labour is documented;
- `Position`, documented.

Adding a state without an entry fails to compile. This is the RFC-063 pattern.

**R-2 — chip selected.** Shape: Q-1.

**R-3 — correct RFC-045's phrase**, and state the variant/state division of
labour in `accessibility.md` (architect's).

## Open questions

**Q-1 — what does a selected chip carry?**

| Option | Change | Consequence |
|---|---|---|
| **(a) Assert luminance, document the caller's mark** | Assert selected vs unselected fill ≥ 3.0:1 (it is 6.19+ today); document "put a mark in the label if your record needs a non-colour cue" | No appearance change. Meets the commonly used 3:1 state-difference reading. Does **not** meet tekstide's word-or-shape bar |
| (b) snora draws a shape | e.g. a leading ✓ glyph when selected (lucide `Check` under the feature, text otherwise) | Meets tekstide's bar for every caller. An appearance change, and it collides with callers such as orbok who already add their own ✓ |
| (c) (a) now, (b) opt-in | `filter_with_mark(..)`-style additive API | Additive; one more API |

**Suggest (a)**, with (c) recorded as the path if a caller asks. orbok already
solves it in the label, and a default-on glyph would double theirs. This mirrors
RFC-093 Q-1's reasoning, stated here rather than assumed.

**Q-2 — are hover, pressed and disabled "states that matter"?** Suggest:
**disabled yes**, since it changes what the control does; **hover and pressed
no**, being transient feedback on a pointer the user is already moving. Record
disabled's cue (alpha today) in the register, and assert it.

**Q-3 — the breadcrumb leaf.** Position is a legitimate cue, but "same colour at
rest as its links" also means ancestors show **no resting affordance**. That is
a different question (are they recognisable as links?), out of scope here.
Suggest: register the leaf as `Position`, and file the link-affordance question
separately.

## Acceptance criteria

1. The state register exists, exhaustive over an enum of snora-drawn states, and
   a new state without an entry fails to compile.
2. Each `Luminance` / `Shape` cue is asserted, and **shown failing** under a
   scratch edit (e.g. the chip's selected fill set to the unselected colour).
3. The chip decision as ruled in Q-1, documented in `accessibility.md`.
4. RFC-045's phrase corrected (architect's).
5. No change to `snora-design` / `snora-style` (quote the diff).

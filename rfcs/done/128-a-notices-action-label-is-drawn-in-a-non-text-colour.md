# RFC-128 — A notice's action label is drawn in a colour that is not a text colour

**Status.** Done — shipped v0.62.0. Accepted 2026-11-14. Every open question ruled as suggested (see Rulings).
**Handoff.** [`handoffs/128-…`](../handoffs/128-a-notices-action-label-is-drawn-in-a-non-text-colour/implementation-handoff.md).
**Raised.** 2026-11-14, architect, from the 0.62 audit's follow-up (Unit 2, H-5), which measured
it and stopped, as its handoff said to.
**Release target.** 0.62.0.
**Priority.** A contrast defect in drawn text, on four of six presets, including both presets
whose whole purpose is AAA text.

---

## The finding

`Notice::action` draws its label in the notice's **tone colour**
(`crates/snora-widgets/src/design/notice.rs`, `text(act.label)…color(accent_color)`). It overrides
the ghost style the action otherwise has. The label is 14 px regular, not large text, on
`palette.surface`.

**Measured by Unit 2** (`snora_design::contrast::contrast_ratio`, every tone × every preset):

| preset | Info | Success | Warning | Danger | Accent | Neutral |
|---|---|---|---|---|---|---|
| light | 6.19 | 4.63 | 5.01 | 6.03 | 6.19 | **3.12** |
| dark | 6.24 | 9.92 | 10.41 | 6.23 | 6.24 | **3.50** |
| light_aaa | 7.05 | **6.51** | **6.51** | **6.51** | 7.05 | **3.12** |
| dark_aaa | 7.81 | 9.92 | 10.41 | **6.54** | 7.81 | **3.50** |
| high_contrast_light | 10.25 | 8.38 | 8.42 | 8.85 | 10.25 | 21.00 |
| high_contrast_dark | 11.75 | 13.41 | 14.85 | 10.34 | 11.75 | 21.00 |

**Two failures, one cause:**
1. **`Neutral` fails AA** (3.12 and 3.50 against 4.5) on Light, Dark and both AAA presets. Neutral's
   tone colour is `palette.border`, a non-text colour held to 3:1.
2. **The AAA presets break their own promise.** `aaa-presets.md`: *"every declared text pair at
   7:1"*. Success, Warning and Danger labels read 6.51 or 6.54 there.
   - Unit 2 tested 4.5 only, so these are the architect's reading of the same table.

**The cause:** the pair "tone colour as text on `surface`" is drawn but never **declared**, so
neither the palette's contrast suite nor the AAA sweep ever checks it. A tone colour is declared
for what it is meant to be: a bar, a border, a fill. Using it as text is the defect.

## Proposal

**R-1. A notice's action is a ghost action, drawn as every ghost is:** its label in the ghost
style's own colour, `palette.accent`, whatever the notice's tone. Remove the override. The tone
still shows in the notice's accent bar and border, which are non-text and held to 3:1.
- `palette.accent` on `surface` is a declared text pair, so the existing suites already hold it:
  6.19 and 6.24 in Light and Dark, 7.05 and 7.81 in the AAA presets, 10.25 and 11.75 in the
  high-contrast ones.
- It is also RFC-117's rule: **an action does something where the reader is, and is a ghost.** A
  notice's action is one.

**R-2. A guard for the class of defect, not only this instance.** A test draws every text a design
widget draws with a colour of its own, and checks it against the text floor of each preset: 4.5,
or 7 on the AAA presets.
- The audit found one such override. A source search finds no other text drawn in a non-text
  colour (`chip.rs` colours a glyph and borders; `progress.rs` uses `text_primary`).
- The test is what keeps it that way.

**R-3. The book:** `notices.md` says that the action is a ghost, in the accent colour, for every
tone, and the sentence the audit narrowed is restored to a true, tested one.

## Open questions, with suggested rulings

**Q-1 — which colour for the label?**
- **Suggest the ghost's accent (R-1).** It is one rule ("an action is a ghost") already in the
  book. It is a declared text pair in every preset. It changes no preset colour.
- **The alternatives, rejected:**
  - `text_primary` would also pass, but makes the action look like body text, so a reader may
    not see it as pressable.
  - Per-tone text colours (`success_text`, …) would add five token fields, frozen-surface
    additions under RFC-036, to keep a tint that the bar and border already carry.
  - Keeping the tone colour and fixing the presets would move `border`, a non-text colour, to
    text strength. That defeats what `border` is.

**Q-2 — is this a breaking change?** **Suggest no.** No API changes. It changes what a notice with
an action draws, so it goes under **Changed** with a migration-guide row (retake screenshot
baselines of notices with actions), and it ships in the same release as the sheet's width.

## Review against snora's principles

- **Nobody is confused.** An action looks like every other action, and its label can be read.
  The tone keeps its cue in the bar and border, as for a notice without an action.
- **Safe and robust.** The defect class gets a guard (R-2), so the next override is caught in CI,
  not by an audit.
- **Clean.** It removes code (one override), and adds no token.
- **The initial cost is not the deciding factor.** The cheapest fix, changing Neutral alone,
  would leave the AAA presets below their promise.
- **ABDD.** No direction is involved.
- **Keyboard.** Unchanged. The action keeps its ring and its keys.

## Acceptance criteria

1. **Unit 0** measures R-1's label against `surface`, idle, hovered and pressed (the ghost's tint),
   every tone × every preset, at the preset's floor. It also measures R-2's sweep on today's
   `main`, which must fail on the four Neutral cells and the AAA cells above.
2. **Unit 1:** R-1 and R-2, each with a breakage row (put the override back).
3. **The book,** by the architect (R-3).

## Rulings, 2026-11-14 (owner)

The RFC is accepted as written:
- **Q-1:** the action's label is the ghost's own colour, `palette.accent`, for every tone.
- **Q-2:** not breaking: **Changed**, with a migration-guide row, in 0.62.0.

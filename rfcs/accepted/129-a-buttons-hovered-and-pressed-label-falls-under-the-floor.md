# RFC-129 — A button's label, hovered or pressed, can fall under its preset's text floor

**Status.** Accepted 2026-11-14. Every open question ruled as suggested (see Rulings).
**Handoff.** [`handoffs/129-…`](../handoffs/129-a-buttons-hovered-and-pressed-label-falls-under-the-floor/implementation-handoff.md).
**Raised.** 2026-11-14, architect, from RFC-128's Unit 0, which measured it.
**Release target.** 0.62.0, with RFC-128.
**Priority.** Contrast in states a reader sees. Most of it is in the presets whose purpose is AAA
text, and one cell is under AA on the default Dark preset.

---

## The finding

Every `design::button` variant, and the ghost a notice's action becomes under RFC-128, changes
colour when hovered and when pressed (`crates/snora-style/src/button.rs`):

| Variant | Hovered | Pressed |
|---|---|---|
| `primary`, `danger` | the fill `lighten(…, 0.06)` | the fill `darken(…, 0.06)` |
| `secondary` | a surface shift | a surface shift |
| `ghost` | `accent` at 8% behind the label | `accent` at 14% behind, and the label `darken(accent, 0.06)` |

**The shifts go one way on every preset.** On a light preset, lightening a fill under white text
lowers its contrast. On a dark preset, darkening a light label lowers it too. Measured by RFC-128's
Unit 0 (`snora_style::button::*` with each status, on the page background, every preset; floor 4.5,
7 on the AAA presets):

| preset | below the floor |
|---|---|
| `light_aaa` | **hovered:** primary 6.23, secondary 5.47, ghost 6.71, danger 5.90 |
| `dark_aaa` | **pressed:** secondary 5.63, ghost 5.63, danger 5.93 |
| `dark`, on `surface` (a notice's ghost after RFC-128) | **pressed:** ghost 4.16, under AA's 4.5 |

Light, Dark (on the page), and the two high-contrast presets pass in every state.

**The cause is the same as RFC-128's:** a hovered or pressed label is text a reader sees, but only
the resting pair is declared. So no suite checks it, and the AAA presets' promise, *"every declared
text pair at 7:1"*, holds only at rest.

## Proposal

**R-1. A state never takes a label under its preset's text floor, in any variant, on any neutral
surface it is drawn on** (the page `background`, `surface`, `surface_raised`).
- **The direction of a shift follows the preset, not a fixed lighten or darken:** a state moves the
  fill or the label **away from** the other, so contrast holds or rises.
- **The shift stays visible:** a hovered or pressed control still draws differently from one at
  rest. A state that keeps its contrast by not changing at all would hide the hover cue.

**R-2. The states are declared.** The contrast sweep covers every variant × every state (Active,
Hovered, Pressed) × every neutral surface × every preset, at the preset's floor. It is the sweep
RFC-128 R-2 begins, extended to every state.
- **The AAA presets' promise then holds in every state,** and `aaa-presets.md` says so.
- **Disabled stays exempt,** as WCAG 1.4.3 exempts inactive controls.

**R-3. Chips' hovered and pressed fills** (`chip.rs`, `darken(surface)`, `darken(accent)`) are in the
same sweep. RFC-128's Unit 0 named them as unmeasured.

## Open questions, with suggested rulings

**Q-1 — how is the direction chosen?** **Suggest: from the measured contrast, not from the preset's
name.**
- Of the two shifts, lighter and darker, a state takes the one that keeps the label at or above its
  floor, preferring the one it uses today.
- Then no preset needs a flag saying "I am dark", and a preset an application builds field by field
  (`Tokens` is `#[non_exhaustive]`, but its fields are public) is handled the same way.

Unit 0 measures this rule, and any simpler one that also holds, and proposes one.

**Q-2 — what does this change for the four presets that pass today?** **Suggest: nothing that
passes is moved.**
- On Light, Dark and the two high-contrast presets, every state already passes on the page. They
  keep their current colours, because the rule only changes a shift that would cross the floor.
- The one exception is Dark's pressed ghost on `surface`, which fails.

So the visible change is confined to the cells that fail, and the migration guide lists them.

**Q-3 — the hover cue.** **Suggest a measured minimum:** a hovered or pressed control differs from
its resting frame in its fill or its label, by a step Unit 0 proposes (for example, the 3:1
non-text floor between the two fills is too much, while a visible step is enough). The guard is
"the state still looks different", tested by frames.

## Review against snora's principles

- **Nobody is confused.** A label stays readable while the reader points at it or presses it. That
  is the moment the reader is looking at it.
- **Predictable.** One rule for every state of every variant: never under the floor, always
  visibly different.
- **Robust.** The states are declared and swept, so a new variant or preset cannot reintroduce it
  silently.
- **The initial cost is not the deciding factor.** The cheap fix, raising the AAA presets'
  colours, would move every resting colour to fix a transient state. The state is where the
  defect is.
- **ABDD.** No direction is involved.
- **Keyboard.** Unchanged. The focus ring has its own guarantee.

## What this is not

- Not a change to resting colours or to any preset's palette.
- Not the focus ring (RFC-116's guarantee) or the disabled state (exempt).
- Not the unstyled path: its colours are the application's iced `Theme`.

## Acceptance criteria

1. **Unit 0:**
   - Q-1's rule measured against every variant × state × neutral surface × preset;
   - the frames that change, listed;
   - Q-3's visibility step proposed.
2. **Unit 1:** R-1 and R-2, with the sweep failing on today's code (the breakage row), and a frame
   test that every state still differs from rest.
3. **The book,** by the architect: `aaa-presets.md` (every state), `buttons.md`.

## What Unit 0 found (2026-11-14)

**Wider than the finding above,** which measured one surface and no chips:
- **36 cells in 15 groups** fall under the floor. A group is a preset, a variant and a state.
- **Two groups are on the default presets:** `light` secondary hovered on `surface` (4.47), and
  `dark` secondary and ghost pressed on `surface` and `surface_raised` (4.16, 3.73). So this
  RFC's "Light and Dark pass in every state" was wrong.
- **Both chips fail** on the AAA presets.
- **A hover cue that does not exist:** on `high_contrast_dark` the unselected chip's hovered and
  pressed states draw **exactly** like rest. Its `surface` is black, so darkening it does nothing.

**Q-1's rule holds,** applied on the worst of the three surfaces, since a style cannot know which it
is drawn on. With Q-3's visibility step (ΔE76 ≥ 3.0):
- every group clears its floor;
- **no group that passed today moves;**
- **17 groups change** (the migration list). They are the failing ones, plus the invisible chip.

**Q-2 holds as ruled:** nothing that passes moves. The visible change is wider than the RFC
expected because more cells fail, not because the rule moved passing ones.

**One question Unit 0 raised that the RFC did not ask (Q-4, below):** the rule needs each
preset's text floor, and `Tokens` does not carry it.

## Rulings, 2026-11-14 (owner)

The RFC is accepted as written:
- **Q-1:** the direction of a state's shift is chosen from the measured contrast, preferring
  today's; Unit 0 proposes the rule, or a simpler one that also holds.
- **Q-2:** nothing that passes today is moved; the visible change is the failing cells, listed in
  the migration guide.
- **Q-3:** a state stays visibly different from rest, by a step Unit 0 proposes, guarded by frames.

## Q-4 — where does a style get its preset's text floor? (raised at Unit 0's review, for the owner)

The rule must know whether a preset holds its text to 4.5:1 or 7:1. Today only the tests know:
`Palette::usages()` is test-only, and the AAA presets are named in the sweep.

**Suggest a token: `Tokens::text_contrast`, an enum, `TextContrast::Aa` or `TextContrast::Aaa`.**
- **Each preset says what it promises,** in data a reader can see:
  - `light` and `dark`: `Aa`;
  - `light_aaa` and `dark_aaa`: `Aaa`;
  - the two high-contrast presets: `Aaa`, since their text is 8.38:1 or more, beyond AAA.
- **The suites read it too,** instead of naming presets. A preset an application builds field by
  field chooses, and `Aa` is the default.
- **An enum, not a number,** so no preset can say "5.3", and the book's table and the token say the
  same thing.
- It is an addition to the frozen design surface (RFC-036), as `targets` was in 0.55: `Tokens` is
  `#[non_exhaustive]`, and every preset sets it.

**Rejected:**
- **Deriving the floor from each pair's resting contrast** holds 14 pairs to 7 where the preset says
  4.5. It moves a `light` chip that passes today: a rule about one preset changing another,
  which no reader could predict.
- **One floor of 4.5 for every preset** leaves the AAA presets, half the finding, unfixed.

**What it changes for the high-contrast presets:** their states are then held to 7:1. Unit 0
measured them passing at 4.5, and Unit 1 measures them at 7. Any cell under 7 joins the
migration list by the same rule.

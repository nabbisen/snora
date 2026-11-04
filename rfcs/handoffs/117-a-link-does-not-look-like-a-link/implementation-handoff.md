# Implementation handoff — RFC-117, Unit 0 and R-5: measure the underline, declare the accent

**RFC.** `rfcs/accepted/117-a-link-does-not-look-like-a-link.md`
**Scope.**
- **Unit 0:** a measurement, delivered as its own review. The owner then rules Q-5,
  and `link` itself gets a follow-up handoff.
- **R-5:** small, and lands on `main` now.

**Order.** After RFC-113 and RFC-114 are committed, since R-5 touches the shared
CHANGELOG. RFC-118 depends on R-5.

## R-5 — declare the accent's text use (`snora-design`, test code only)

**Already established:**
- `style::button::ghost` draws its label in `palette.accent` (`snora-style/src/button.rs:138`);
- `Palette::usages` declares `accent` only as `ThresholdClass::NonText`
  (`snora-design/src/palette.rs`, the RFC-063 comment);
- measured today, the accent clears 4.5:1 on every neutral surface of every preset.
  The lowest is 5.63, Dark `surface_raised`.

**Do:**
- **A second `accent` entry** in `usages`: `ThresholdClass::Text` on
  `neutral_surfaces()`, with a comment naming the ghost and link labels.
- **Keep the existing `NonText` entry,** which covers the filled primary button.

**Show failing:** set the Light accent temporarily to a colour at about 4.0:1 on
`surface`. The suite must fail **on the new text entry**, and not only pass the
non-text entry. Then restore it.

**CHANGELOG** under Fixed: the accent's text use is now declared and guarded.

## Unit 0 — how `rich_text`'s underline renders (no source change)

Render, on **both renderers**, through RFC-108's `frame_pixels` route:
- a one-line and a two-line `rich_text` with one span, `underline(true)`, in
  `palette.accent` on `background`;
- in all four presets, at the default text size and at a host text size of 24 px.

**For each, measure:**

| Measure | Meaning |
|---|---|
| present per line | an underline under **each** line of the two-line case |
| thickness | rows of underline pixels, at the simulator's scale 2 |
| gap | rows between the glyphs' lowest ink and the underline |
| contrast | underline core against `background`, ≥ 3:1 |
| renderer parity | the same figures on wgpu and tiny-skia, or the difference stated |

**For comparison,** the same figures for RFC-109's 1 px crumb rule, which is already
measured.

**Report:** the table, plus your recommendation on Q-5, `rich_text` or the rule
element.
- If you recommend the rule element, say what a wrapped label would then show.
- **Remember** that Unit 0 compares underlines on two renderers, which is not a
  before/after claim (see RFC-108's parity note).

## Acceptance criteria

1. R-5 shown failing, then passing, and committed.
2. The Unit 0 table complete on both renderers, with the probe source in the evidence.
3. No change under `crates/` for Unit 0.

# Implementation handoff — RFC-117: `link`, drawn by snora, underlined per line

**RFC.** `rfcs/done/117-a-link-does-not-look-like-a-link.md`. Read its "Ruling,
2026-11-07 (Q-5)".
**Release target.** 0.55.0, with RFC-116 R-1.
**Order.** After RFC-116's Unit 2 is accepted, since `link` is built on
`snora-control`.

## Unit 0b — prototype, measure, propose the rule

A prototype `link` widget, on the branch:
- **it owns its paragraph,** laid out from the label;
- **one rectangle per line,** read through `Paragraph::span_bounds(0)`
  (`iced_core/src/text/paragraph.rs:66`, implemented in `iced_graphics`, so both
  renderers share it);
- **an underline drawn under each rectangle,** for now at a thickness and offset
  derived from the text size.

**Measure,** with your Unit 0 probe and on both renderers:
- one-line and wrapped (three lines at 16 px, four at 24 px);
- at 16 px and at 24 px text;
- for each line: present, thickness in physical rows, gap to the glyphs' lowest ink,
  and contrast.

**Then propose one rule,** stated so a reader can apply it by hand. For example:

> *An underline is 1/16 em thick, at least 1 px, and sits 1/16 em below the
> descenders' ink.*

The figures are yours to set from the measurements. **Show what your rule gives at
16, 24 and 32 px,** and the gap at 16 px, where `rich_text`'s was 0. This is its own
review, and the rule is ruled there.

## Unit 1 — `design::button::link(tokens, label, on_press)`

After the rule is ruled:
- built on `control`, so it is focusable, Enter presses it, and the ring draws;
- the ghost style's colours, with the underline drawn in the label's colour, in every
  state;
- **no `link_maybe`.** There is no disabled link (RFC-117 R-2), and the rustdoc says
  what to do instead;
- **R-3's rule** in the rustdoc of both `link` and `ghost`: a link takes the reader
  somewhere, and an action does something where they are.

**Assert:**
- an underline under every line of a wrapped link, at the ruled thickness and gap;
- the underline ≥ 3:1 and the label ≥ 4.5:1 (R-5's text entry) on all six presets;
- keyboard as for every control;
- `ghost` unchanged.

Each is shown failing.

## Acceptance criteria

1. Unit 0b's figures and proposed rule, reviewed and ruled.
2. Unit 1 passing, with each breakage shown failing.
3. CHANGELOG under Added. The architect writes `design/buttons.md`.

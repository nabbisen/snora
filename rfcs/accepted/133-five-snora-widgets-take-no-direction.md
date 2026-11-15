# RFC-133 — Five snora widgets take no direction, so they cannot mirror under RTL

**Status.** Accepted 2026-11-15 (owner), every open question ruled as suggested (see Rulings).
**Handoff.** [`handoffs/133-…`](../handoffs/133-five-snora-widgets-take-no-direction/implementation-handoff.md).
**Raised.** 2026-11-15, architect, from RFC-132's Unit 0, which measured them.
**Release target.** 0.64.0 (Q-3).
**Priority.** ABDD is a requirement. These are defects since the widgets were written, and named as
a known limit in 0.63.

---

## The finding

RFC-132's Unit 0 measured every row snora builds in both directions. **Five public widgets draw a
row with a reading order, and take no `LayoutDirection` at all.** They draw the same under RTL as
under LTR:

| Widget | Signature today | What stays left-to-right under RTL |
|---|---|---|
| `design::notice::Notice` | `Notice::new(tokens, tone, body)`, a builder | the accent bar stays at the left; action, then dismiss |
| `design::chip::removable`, `removable_with_tooltip` | functions | the label, then the × |
| `design::progress::row`, `card` | functions | the label is left-aligned, and **the bar fills from the left** (iced's `progress_bar` has no direction) |
| `app_footer` (unstyled and design) | `app_footer(content)` | the content is anchored at the left edge |
| `render_menu` | `render_menu(menu, on_action, is_active)` | the icon is left of the label (the header's own menus know the direction; a bare `render_menu` does not) |

The header, the sidebar, the tab bar and the breadcrumb already take a direction. These five are the
rest.

## Proposal

**R-1. Every snora widget whose layout has a reading order takes the direction, as a required
parameter.** This is the same as `app_header`, `app_side_bar`, `app_tab_bar` and `app_breadcrumb`
do now:
- `removable`, `removable_with_tooltip`, `progress::row`, `progress::card`, both `app_footer`s and
  `render_menu` each gain a `direction: LayoutDirection` argument;
- `Notice::new` gains it too (Q-1).

**R-2. Each lays itself out with RFC-132's row:** logical order, mirrored positions. **The progress
bar fills from the start edge,** which is right under RTL (Q-2).

**R-3. Tested both ways,** from bounds, with the progress fill read from pixels. A test also lists
every public widget function with a row, and checks each takes a direction, so a sixth cannot be
added without one.

**R-4 (added 2026-11-15 at RFC-132 Unit 1's review, the owner told). snora's own text aligns to
the start edge under RTL.** RFC-132 mirrors the cells, but the text inside a cell stays
left-aligned: a toast's title, for one, now sits in a mirrored cell aligned to its left. Unit 0
measures every snora text's alignment under RTL, and Unit 1 aligns each to the start edge. This
is not text shaping, which is iced's; it is where a text box puts its line.

**R-5 (added the same day). `Shortcuts::routes(&self, direction)` loses its parameter.** No route
depends on the direction since RFC-132, and its rustdoc says so in 0.63. A parameter that does
nothing tells a reader it matters. It is removed in this release, which is already breaking.

## Open questions, with suggested rulings

**Q-1 — a required parameter, or a builder with a default?** **Suggest: required, everywhere,
`Notice::new` included.**
- A default would be `Ltr`, and that is silently wrong in every RTL application. Nobody would
  learn of it until a reader saw it. A compile error tells the application where to look, once.
- It is the shape the other four direction-aware widgets already have, so there is one rule to
  learn.
- **Rejected:**
  - **direction in `Tokens`:** it is application state that flips at run time, not a design
    value, and the frozen design surface is the wrong place for it;
  - **an `Ltr` default:** for the reason above.

**Q-2 — the progress fill under RTL.** iced's `progress_bar` always fills from the left.
**Suggest:** snora's progress draws its bar through RFC-132's mirrored layout, or draws the fill
itself, whichever Unit 0 finds smaller, so it fills from the start edge. If neither is possible in
iced 0.14 without public API, Unit 0 says so.

**Q-3 — 0.64.0.** **Suggest: yes.**
- It changes the signature of seven public functions and one constructor, so every call site
  changes.
- 0.64.0 is already the release that removes the deprecated F6 types. One breaking release,
  announced in 0.63's guide, costs adopters one migration instead of two.
- 0.63's book names the five as a known limit.

## Rulings, 2026-11-15 (owner)

Accepted as proposed:
- **Q-1:** a required `direction` parameter everywhere, `Notice::new` included. No default, and
  not in `Tokens`.
- **Q-2:** the progress bar fills from the start edge, by RFC-132's row or a fill snora draws,
  whichever Unit 0 finds smaller.
- **Q-3:** 0.64.0, with the deprecated F6 removal.

## Review against snora's principles

- **ABDD.** It completes the requirement: every snora widget with a reading order mirrors, and a
  test keeps it so.
- **Nobody is confused.** The parameter says the widget depends on direction, and its absence
  cannot pass silently.
- **Predictable.** One shape for every direction-aware widget.
- **The initial cost is not the deciding factor.** A breaking parameter costs every adopter an edit,
  once. A default would cost every RTL reader a wrong layout, for good.
- **Keyboard.** Tab follows the logical order inside each, by RFC-132's row.

## Acceptance criteria

1. **Unit 0:** the five measured both ways after RFC-132 lands, Q-2's fill prototyped, and the
   list of public widget functions with a row.
2. **Unit 1:** R-1 to R-3, each with a breakage row; the CHANGELOG `Changed` entry, with the
   one-line migration for each signature.
3. **The book,** by the architect: `direction.md`'s widget table, and the 0.63 → 0.64 guide.

# RFC-138 — A radio group cannot be laid out in a row

**Status.** Accepted 2026-11-26 (owner), every open question ruled as suggested (see Rulings).
**Raised.** 2026-11-26, architect, from logolig's letter of 2026-11-19 (F-2 in
`.git-exclude/upstream/receive/0.65.0/architect-triage-2026-11-26.md`). Scoped into 0.66 by the owner on 2026-11-26.
**Release target.** 0.66.0, as an addition.
**Priority.** Not a defect. A migration from iced's controls must not take a layout away, and a narrow window must not
waste its height.

---

## The finding

- **snora's `radio_group` (0.65, RFC-134) is a column, and nothing changes that.** Its options are laid out by
  `column(rows)` in `snora-widgets/src/form.rs` (`From<RadioGroup>`).
- **iced's radios are separate widgets,** so an application lays them out as it likes, in a row included. An
  application that moves from iced's `radio` to snora's group for its keyboard route therefore **loses** the row. logolig
  hit exactly this: two or three short options ("View as", "Surface") that would sit in one line.
- **The keyboard is not the issue.** The group is one Tab stop, and ↑ ↓ ← → move and select (← → following the screen,
  RFC-132 R-5). Those keys do not depend on the layout.

## Proposal

**R-1. One setting: `horizontal(bool)`,** on `snora::radio_group` and `snora::design::radio_group`.

```rust
radio_group(["List", "Grid"], Some(&view), Message::ViewAs, direction)
    .horizontal(true)
```

- **The column stays the default.** `horizontal(false)` is today's group, unchanged.
- **The argument is a `bool`,** as `disabled(bool)` on the same builder is, so a responsive layout can pass its own
  condition (`.horizontal(window_is_wide)`) with no `if` around the builder.
- **The name is not "direction",** which on this builder is the reading direction (LTR or RTL). Horizontal or vertical is
  a different question, and it gets a different word.

**R-2. A horizontal group is a row that wraps.**
- **The options keep their logical order, line by line.** When the group is wider than the room it has, it continues on
  the next line, and an option is never clipped or squeezed (RFC-105).
- **Under RTL the group is mirrored line by line:** the first option at the right end of the first line, as a reader of
  right-to-left text expects.
- **The options sit at the start edge of each line,** as the column's sit at the start edge of the group.

**R-3. The keys do not change.**
- One Tab stop, with ↓ and → moving to the next option and ↑ and ← to the previous one, wrapping at the ends. ← → follow
  the screen.
- This is the platform convention for a radio group (WAI-ARIA's radio group pattern), and it is the same in a row and in
  a column. **There is no two-dimensional movement in a wrapped row:** ↓ is "next", not "the option below". The book says
  so.

**R-4. Every floor holds in the new layout,** as in the column:
- the ring around the group at 3:1 (RFC-116);
- each option's pointer target at least 24 px (RFC-115);
- labels at the palette's level, and the mark at 3:1 against the page and the row's tints (RFC-131, RFC-135);
- a click on the selected option, and Space or Enter, publish nothing (RFC-134 ruling 3).

The drawn sweeps take the horizontal group in.

## Open questions, with suggested rulings

**Q-1 — the shape: `horizontal(bool)`, or an enum such as `orientation(Orientation::Horizontal)`?** **Suggest
`horizontal(bool)`.**
- No other snora widget switches orientation, so an enum would be a new type with one user.
- `bool` matches `disabled(bool)` on the same builder.
- `.horizontal(true)` reads one way only.

**Q-2 — the space between options in a row.** **Suggest: Unit 0 measures the candidates and proposes one.** It is a token
on the design path (the literal-size gate forbids a literal), and one constant on the engine path. It must:
- separate one option's label from the next option's mark more clearly than a mark from its own label, so that a label
  is never read as belonging to the option after it;
- keep the space between lines the column's.

**Q-3 — does it wrap by itself, or does the application choose wrapping?** **Suggest: it always wraps.** A row that
overflows clips an option, which RFC-105 rules out. A setting to turn wrapping off would be an opt-in to a defect.

**Q-4 — when.** **Suggest: 0.66.0,** with H-1 (the toast-target tolerance) in the same unit or beside it.

## Rulings, 2026-11-26 (owner)

"Yes." Every question as suggested:
- **Q-1:** `horizontal(bool)`. No orientation enum.
- **Q-2:** Unit 0 measures the candidates for the space between options and proposes one, to the rule above. It is a
  token on the design path and one constant on the engine path. The architect rules on the proposal at Unit 0's review.
- **Q-3:** a horizontal group always wraps. There is no setting to turn wrapping off.
- **Q-4:** 0.66.0, as an addition.

**Unit 0's review, 2026-11-26 (architect).**
- **Q-2 ruled: the space between options is twice the space between a mark and its label,** on both paths: 16 px on
  the engine path (`2 × 8`) and `2 × tokens.spacing.sm` on the design path (16 with the stock tokens). It is derived
  from a token rather than taken from `spacing.lg`, because the rule is relative and `Spacing`'s fields are public: an
  application's scale with `lg` at or below `sm` would break it silently. Measured: an ink ratio of 1.8 to 1.9 against
  the mark-to-label gap, where 8 px gives 0.9 to 1.0.
- **R-2's "never clipped or squeezed" has a limit, measured in Unit 0.** A line breaks between options, so no option is
  pushed out of the room by the ones before it. An option that alone is wider than the room is laid out as the column
  lays it out, which is squeezed (100 px window, both paths). That limit is the column's too, so it is carried into
  0.67 to be measured on its own, and the doc and the book state it.

## Review against snora's principles

- **Nobody is confused.**
  - The name cannot be read as the reading direction.
  - The keys are the same in either layout, and the platform's.
  - The default is today's.
- **Clean.** One setting, and no second constructor or second widget.
- **Robust.** It always wraps, so nothing is ever clipped. Every floor is measured on the drawn pixels.
- **ABDD.** The order is logical and the layout mirrors under RTL, line by line, as RFC-132 rules for every row.
- **Keyboard.** Unchanged: one stop, arrows that select.
- **Correspondence cost.** None now. One line in 0.66's news to logolig.

## Units

**Unit 0: a prototype, measured.**
- Two, three and six options, at a wide window and a narrow one, LTR and RTL, both paths:
  - where each option is drawn, line by line;
  - each option's pointer target;
  - the ring around the group;
  - the keys (`every_control.rs`'s probe);
  - what is clipped (nothing may be).
- Q-2's candidates for the space between options, drawn.

**Unit 1: the implementation.**
- R-1 to R-4 on both paths.
- The drawn sweeps (`text_sweep.rs`, `nontext_sweep.rs`) take a horizontal group in.
- A mirrored-layout test: RTL is the mirror image of LTR, line by line.
- A narrow-window test: no option is clipped.
- Each test gets a breakage row, among them: no wrap; no mirroring; the column's layout ignored the setting.
- CHANGELOG `[Unreleased]`, Added.

## Acceptance criteria

1. **Unit 0:** every case above measured, and Q-2's proposal.
2. **Unit 1:** R-1 to R-4, each guard shown able to fail, and every `ci.yaml` command run locally.
3. **The book,** by the architect: the keyboard guide's form-control table and the radio group's line on the keys.

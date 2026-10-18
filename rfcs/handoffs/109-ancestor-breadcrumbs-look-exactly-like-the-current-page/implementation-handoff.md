# Implementation handoff — RFC-109: ancestor breadcrumbs look exactly like the current page

**RFC.** `rfcs/accepted/109-ancestor-breadcrumbs-look-exactly-like-the-current-page.md`
**Release target.** 0.53.0.
**Rulings.**
- Q-1: an underline.
- Q-2: a rule element under the label, inside the button, snapped to whole pixels.
- Q-3: no opt-out.
- Q-4: 0.53.0.

**Touch list.**
- `crates/snora-widgets/src/crumb.rs`;
- `crates/snora-widgets/src/state_register.rs`;
- a rendered test under `crates/snora/tests/`;
- `CHANGELOG.md` (`[Unreleased]`).

**Not** `snora-style` or `snora-design`. **Not** the label colour.

## Already established

- **Resting colours are the same by construction.** An ancestor is a `button` whose
  resting arm of `crumb_button_style` sets no background and
  `text_color: palette.background.base.text`. The leaf is plain `text`, which
  inherits `theme.base().text_color`, and that is the same value (`iced_core`
  `theme.rs:334`).
- **Both paths share the style.** The design variant (`design/widget.rs`,
  `breadcrumb_geometry`) passes geometry only, and shares `crumb_button_style`.
- **Hover** fills `background.weak.color`. Keep it.
- **The register** has `BreadcrumbLeaf => Cue::Position`, and nothing for ancestors.
- **Snap the rule.** RFC-108 measured a 1 px rule at a fractional y position smeared
  over three rows by tiny-skia (about 50% at the edges). The new rule must not be
  placed that way: whole-pixel offset, 1 px minimum height.
- **The tab underline is the precedent to copy:** a rule element, asserted by
  rendered bounds (RFC-102 and RFC-103, `tests/tab_bar_edges.rs`).

## Unit 1 — failing-first (register)

1. Add `State::BreadcrumbAncestor => Cue::Shape { asserted_by: <the new test> }`.
2. The new rendered test asserts that **every ancestor has a rule of ≥ 1 px under
   its label and inside its button bounds, and the leaf has none**. Locate the
   ancestor by clicking (`pointer_target`), not by tree position.
3. **Today it fails**, since there is no rule. Record that.

## Unit 2 — the rule

- Inside the ancestor's button content, put the label and then a 1 px rule element
  under it, in the label's resting colour (`background.base.text`), spanning the
  label's width.
- **Choose the geometry** so the rule's top edge lands on a whole logical pixel at
  both the unstyled and the design label sizes (16 and 14 at defaults), and show the
  numbers. If the padding cannot keep it whole, round the rule's offset, not its
  height.
- On **hover and press** the rule stays, and the fill appears as today. The colour
  follows `crumb_button_style`'s hovered arm, `background.weak.text`, so the rule
  matches the label in both states.

## Unit 3 — measured

On the unstyled path (Light, Dark) and the design path (all four presets), record:

- rule bounds per ancestor;
- the rule's colour against the page, **≥ 3.0:1**, asserted in the rendered test,
  or in a style test if the colour comes from a style function;
- the crumb row's height before and after, for the migration guide;
- the pointer target before and after, which must not shrink (`pointer_target`).

## Check each assertion can fail

Back up to named files with `cp`, break, run, restore, then `sha256sum -c`. Never
`git checkout`/`restore`.

| Assertion | Breakage |
|---|---|
| a rule under every ancestor | remove the rule |
| no rule under the leaf | add one to the leaf |
| rule contrast ≥ 3.0 | colour the rule `background.weak.color` |
| pointer target not smaller | shrink the button's padding by 2 px |

**If a breakage cannot make its row fail, replace the assertion and say so.**

## Acceptance criteria

1. Units 1–3, with the table's breakages shown failing.
2. `render_semantics.rs` unmodified, and the register's `states_without_a_cue` still
   empty.
3. CHANGELOG under Changed: an appearance change on the default path, ancestors now
   underlined, with the row height before and after.

# Implementation handoff — RFC-103: the channel register covers variants, not states

**RFC.** `rfcs/accepted/103-the-channel-register-covers-variants-but-not-states.md`
**Release target.** 0.52.0. **Independent** of RFC-104 and RFC-105; it may run
in parallel. Do not stage their files.
**Rulings.** Q-1 (a): assert the chip's selected/unselected difference at
≥ 3.0:1 and document the caller's mark. Q-2: disabled counts, hover and pressed
do not. Q-3: breadcrumb leaf registered as `Position`.
**Touch list.** A new state-register test module in `crates/snora-widgets/src/`,
`crates/snora-widgets/src/lib.rs` (the `mod` line), `CHANGELOG.md`. **Not**
`docs/`: the `accessibility.md` division-of-labour text and RFC-045's phrase are
the architect's. **Not** `snora-design` / `snora-style`.

## Already established

- RFC-093's register, `crates/snora-widgets/src/design/channel_register.rs`,
  covers **variants** (notice, progress) and uses `assert_colour_only`. The
  toast has its own register, `crates/snora/src/toast/channel_register.rs`. Model
  the new module on RFC-093's shape, not on its assertion: states assert that a
  cue **exists**, where variants assert that only colour varies.
- **Measured:** the chip's selected fill (`palette.accent`) against its
  unselected fill (`palette.surface`) is **6.19 / 6.24 / 10.25 / 11.75:1**
  (light / dark / hc_light / hc_dark). The chip's styles are
  `chip_style_selected` / `chip_style_unselected` in `design/chip.rs`, and the
  disabled arm is `Color { a: 0.5, ..accent }` or `..surface`.
- The states snora draws, and their cues today:

  | State | Cue | Already asserted by |
  |---|---|---|
  | Active tab | `Shape` (2 px underline) | `crates/snora/tests/tab_bar_edges.rs`; indicator contrast in `contrast_tests.rs` (RFC-102) |
  | Sidebar active | Fill, `Luminance(3.0)` against the rail | `contrast_tests.rs` (RFC-085) |
  | Chip selected | colour only today → `Luminance(3.0)` | **nothing** — this RFC adds it |
  | Chip / remove disabled | alpha 0.5 | **nothing** — this RFC adds it |
  | Menu open | `Shape` (dropdown rendered) | `render_semantics.rs` |
  | Breadcrumb leaf | `Position` (last, plain text) | documentation only |

## Unit 1 — the register

- **An enum of every snora-drawn state**, and a function mapping each to its cue
  by **exhaustive `match`**, so a new state without an entry fails to compile
  (RFC-063's pattern). Cue kinds: `Shape { asserted_by: &str }`,
  `Luminance { min: f32 }`, `CallerText`, `Position`.
- **Find the states; do not trust the table above.** Grep for `is_active`,
  `selected`, `is_leaf` and `Status::Disabled`, and check **which controls can
  actually be disabled**. Chip and remove can (`on_toggle` / `on_remove` take
  `Option`). Tab, sidebar, crumb and menu buttons always receive an `on_press`,
  as far as I read; confirm it. If any is reachable as disabled, register it.
- `Shape` entries name the test that asserts them. **Check that each named test
  exists and fails without the cue.** For the tab underline, RFC-102's evidence
  already shows this; cite it. For the menu, find the test and show it.

## Unit 2 — the assertions this RFC adds

- **Chip selected:** selected fill against unselected fill ≥ 3.0:1, all four
  presets, `Status::Active`. **Show it failing** with the selected fill set to
  the unselected colour in a scratch edit (it drops to 1.00:1).
- **Disabled (chip, remove):** composite the disabled fill over the page
  (`contrast::composite_over`) and compare it with the enabled fill. **Measure
  first**, then assert a floor at the measured minimum rounded **down** to one
  decimal, and record the measurement in the test's comment. **Show it failing**
  with the disabled alpha set to `1.0`. A bare "differs" assertion is not
  enough: it passes on a one-bit difference.
- Every cue assertion runs in the six widget-suite contexts where the widget
  takes a `Theme`, and in the four presets where it takes `Tokens`, following
  RFC-102's precedent for `card_raised`.

## Check each assertion can fail before writing it

Three handoff tests in a row could not fail. For each assertion above, state the
scratch edit that makes it fail, run it, and put the output in the evidence. If
an assertion I specified cannot fail, replace it and say why.

## CHANGELOG

**Changed** (a test and documentation change, not a behaviour change): the
channel register now covers states; the chip's selected state is asserted at
≥ 3.0:1 luminance against unselected, and disabled against enabled. Name it as
REQ-004 in the consumer requirements register, stated by tekstide.

## Acceptance criteria

1. The state enum is exhaustive, and a new variant fails to compile without an
   entry.
2. Chip-selected and disabled assertions exist and are **shown failing** under
   the stated scratch edits.
3. Every `Shape` entry names an existing test, **shown failing** without its
   cue (RFC-102's evidence may be cited for the tab).
4. The disabled floor is measured and recorded in the test comment.
5. The covenant diff is quoted and empty; nothing under `docs/`.

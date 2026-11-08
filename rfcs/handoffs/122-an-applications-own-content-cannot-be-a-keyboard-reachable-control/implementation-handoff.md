# Implementation handoff — RFC-122: `snora::control` and `snora::design::control`

**RFC.** `rfcs/accepted/122-an-applications-own-content-cannot-be-a-keyboard-reachable-control.md`.
Read R-1 to R-3 and the Rulings.
**Release target.** 0.58.0. **Unit 0 first,** as its own review. Then Unit 1.
**Branch.** `rfc122-control`, off `main`. Land as `main` requires:
1. rebase onto `main`;
2. push the rebased commits to a new branch name;
3. wait for CI green there;
4. fast-forward with `git push origin <sha>:main`.

## What is being made

An application's own pressable content, made keyboard-reachable. aaai asked for it; RFC-116
L-4 planned it. The shape is iced `button`'s, so an application changes one name:

```rust,ignore
snora::control(row![glyph, text("Open")])            // engine path
    .on_press(Message::Open)
    .padding([6, 12])
    .style(move |theme, status| my_style(theme, status)); // iced's button::Status / button::Style

snora::design::control(&tokens, content)              // design path: ring, floor, default style from tokens
    .on_press(Message::Open);
```

**The public builder** (Q-2):
- `on_press`, `on_press_maybe` (where `None` is disabled and skipped by Tab) and
  `on_press_with`;
- `padding`, `width`, `height` and `clip`;
- `style(Fn(&Theme, button::Status) -> button::Style)`. **Nothing else.**

**Built in, with nothing to set:**
- a focus target, with Tab through `traverse`;
- Enter and Space press it, once (RFC-121 R-3);
- a click presses it and does not focus it, with Tab carrying on after it (RFC-121);
- a ring at ≥ 3:1 (RFC-116);
- the target floor `max(24, 1.5 × text, token)` (RFC-115);
- out of reach beneath an overlay (RFC-119);
- R-1 of RFC-120 when it goes away. An undeclared multi-control removal leaves nothing
  focused.

**Not public** (R-3): the internal `snora_interaction::Control`, its tag, its span
declaration (*future wanted*), `min_target`, `text_size`, the ring's internals, and
`on_focus_change`. The public type wraps the internal one.

## Unit 0 — aaai's three cases, on a prototype

Build a prototype of `snora::control` and `snora::design::control`. Put aaai's cases on it,
on both renderers:

| Case | Shape |
|---|---|
| a glyph beside a label | `row![icon, text("Open")]`, design path with `style::button::primary` |
| a file-tree row, pressable as a whole | a `row` of an indent, an icon and a name, `width(Fill)`, engine and design paths |
| a suggestion whose text is data | `text(caller_string)`, which can be long and must wrap (RFC-105), design path with the default `ghost` |

For each, measure:
- **the keyboard probe of `every_control.rs`:** Tab reaches it, Enter and Space publish
  what a click publishes, and nothing is published while it is unfocused;
- **the ring ≥ 3:1** in the 22 stock themes (engine) and the 6 presets (design);
- **the floor:** 24 (and `1.5 × text`) by default, and 44 with `Tokens::targets.min = 44`.
  A `width(Fill)` row keeps its content at the start, as `control`'s layout does for a
  menu item;
- **no pointer regression:** a click presses it once, draws no ring, and focuses nothing;
- **the default style:** `ghost`'s colours on every preset, and `.style(…)` replacing them.

**Report** anything in R-1 to R-3 that does not fit these cases. One example: if a
file-tree row needs something the builder lacks, say what, and do not add it. It returns
to the owner.

## Unit 1 — the public types

After Unit 0 is reviewed:
- **the public types,** in `snora` (engine) and `snora::design` (design), with rustdoc.
  The rustdoc states what is built in, shows the one-line move from iced's `button`, and
  says it is built on `snora-interaction`, which applications do not depend on;
- **`every_control.rs`** takes both as subjects, so every future keyboard rule applies to
  them automatically;
- **a breakage row per built-in property** (focus target, Enter/Space once, click does not
  focus, ring, floor, beneath an overlay, the default style), each shown failing. Named
  `cp` backups, `sha256sum -c`; check each row can fail first;
- **an example:** extend `examples/starter` or `examples/workbench` with one
  `snora::control` around custom content;
- **CHANGELOG, Added:** both functions, and that this closes RFC-116 L-4.

## Acceptance criteria

1. Unit 0's table, on both renderers, reviewed.
2. Unit 1: every breakage row shown failing, and `render_semantics.rs` unmodified.
3. All of `ci.yaml`'s Rust quality gate commands, the Docs job's commands, the feature
   matrix, the opt-out probe, both rendered jobs and parity. Then the branch CI.
4. **The book is the architect's:**
   - `buttons.md` and `semantic-accessibility.md`;
   - `result-card.md`, where the card becomes a control;
   - the keyboard map;
   - the accessibility guide's "your own buttons" bullet.

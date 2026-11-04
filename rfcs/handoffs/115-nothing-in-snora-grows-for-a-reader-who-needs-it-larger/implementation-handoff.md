# Implementation handoff — RFC-115, Unit 0 only: measure every target and icon

**RFC.** `rfcs/accepted/115-nothing-in-snora-grows-for-a-reader-who-needs-it-larger.md`
**Scope.** Unit 0, a measurement, delivered as its own review request. **No source
change.** R-2's rule, target ≥ max(24 px, k × text), is designed after this reports,
and k is ruled then.
**Order.** After RFC-113 and RFC-114 are committed, since both change what is
measured: the toast's hover wrapper and the labelled rail.

## What to produce

One table, on both paths. Each row is a **pointer target** or an **icon** that snora
draws, and gives:

| Column | Meaning |
|---|---|
| element | e.g. "toast close", "sidebar item (labelled)", "chip dismiss" |
| path | unstyled, or design (one row per preset if they differ) |
| width × height at default text | rendered bounds, logical px |
| width × height at 24 px host text | same, with the host's `default_text_size` at 24, or `Typography` scaled to match on the design path |
| follows | text, tokens, or neither (a literal), with the source line |
| icon size | for icons and glyphs: rendered px at each text size |

**Measure, don't read.** Use `iced_test` bounds by behaviour, as RFC-105 did: find
the element by clicking it or by its text, not by tree position. The RFC's own table
came from a grep and is a starting list only, not the inventory. **Find every
target**, including ones neither the RFC nor this handoff names: tabs, crumbs, chips,
menu items, header controls, dialog and sheet controls.

**Pointer only.** iced 0.14's buttons take no keyboard input (RFC-114's correction),
so "target" means a pointer or touch target. Do not record focus behaviour.

## Also report

- **Every row under 24 px at either text size.** That would be under WCAG 2.5.8's AA
  minimum, a defect today, not a Theme E question. Report it at once; do not wait for
  the table.
- **Every row that does not move between the two text sizes.** These are what R-2
  changes.
- **For each row:** what "k × text" would give at k = 1.5 and at k = 2.75, so the owner
  can see Q-1's two positions side by side.

## Acceptance criteria

1. The table, every row measured, with the probe source in the evidence.
2. A list of targets found that the RFC did not name.
3. No change under `crates/` or `examples/`.

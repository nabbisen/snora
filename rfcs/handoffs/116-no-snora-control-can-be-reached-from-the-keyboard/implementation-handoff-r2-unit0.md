# Implementation handoff — RFC-116 R-2, Unit 0: measure the shortcuts router before it is ruled

**RFC.** `rfcs/done/116-…`: read R-2, and **"Proposed amendment to R-2, for 0.57"**,
including the 2026-11-08 additions B1 to B3.
**Release target.** 0.57.0. **If this unit shows the router does not fit in 0.57, the
owner decides at its review whether it moves to 0.58.**
**Order.** After `housekeeping-0-57` lands, so that the prototype is written against
`snora-interaction`. **Nothing here ships:** this unit is prototypes and measurements, in
test files and a scratch module. The amendment is ruled on what you find.

## What the router is to be, if the measurements allow

The amendment proposes one router, which the application builds in `view` from the same
side bar, tab bar and crumbs it hands its widgets:
- passed with `AppLayout::shortcuts(…)`;
- placed by the engine **inside the page's `Inert`**, so it falls silent under every
  overlay;
- **every route opt-in**, and **every binding with a modifier** (Ctrl, Alt or Cmd). WCAG
  2.1.4 governs single-character shortcuts, so this keeps the router outside it by design.

**Three rules a reader can predict** (the amendment's, plus 2026-11-08's):
- **R-2's own rule:** a shortcut emits exactly the message a click on that control emits.
- **B1:** *a shortcut does exactly what a click on that control does.* That covers the
  message, focus taken away from wherever it was, and the Tab starting point set on that
  control (RFC-121), and nothing when the control is disabled. Without B1, Ctrl+2 would
  swap the view while keyboard focus stays in the old view's tree position. The control
  that moves into that position would then inherit it, and Enter would press something
  nobody chose (RFC-120's finding, by another route).
- **B2:** *a shortcut works unless the focused control uses that key itself.* A code
  editor's Alt+↑ or a terminal's Ctrl+Tab wins.

## Measure

| # | Question | How |
|---|---|---|
| 1 | **Silent under overlays.** With a dialog, a sheet, or a menu with its backdrop open, does a router inside the page's `Inert` answer nothing, and leave the event `Ignored`? | A prototype router widget inside `Inert`, driven through `UserInterface`, for each row of RFC-119's table |
| 2 | **Captured when it answers.** Does it return `Captured`, so that the application's `keyboard::listen()` gets no second message? | The status from `update`, as in RFC-119 Unit 0 |
| 3 | **B2, precedence.** Can the router see a key only after the focused widget has had it? (Delivered after the children, or answering only `Ignored` events.) With a focused `text_input`, and with a focused widget that captures the key, does the router stay silent? | Both orders, with the result recorded |
| 4 | **B1, a shortcut as a click.** From `Inert`, which can run operations in `update` (RFC-120), can the router (a) publish the target control's message, (b) take focus away from the focused control, and (c) set the target's Tab starting point (`start_here`, RFC-121)? Does a disabled target do nothing? | A prototype on `snora-interaction`. Then the view-swap case: focus in view A, Ctrl+2, view B. Show focus is **not** inherited. Also show that it **is** inherited without B1, so the test can fail |
| 5 | **The Escape field.** For an overlay with **no** click-outside close message, can an `AppLayout` field (for example `.escape(Escape { modal, menu })`) answer Escape **on the overlay layer**, as `OnEscape` does, while the overlay is open? What happens when both it and `on_close_modals` are set? Propose the rule | A prototype on the overlay layers, plus a short proposal |
| 6 | **B3, the bindings.** Ctrl/Cmd + 1…9 is the browsers' and most tabbed applications' binding for **tabs**; R-2 gave it to **sidebar items**. Survey the conventions, with sources, on Windows, GNOME and KDE, macOS (Apple's HIG), the major browsers, and one or two widely used desktop applications with both a sidebar and tabs. Do the same for Ctrl+Tab and for the parent crumb (Alt+↑ / Cmd+↑). Propose bindings that would not surprise a reader of those conventions | A table of sources, then a proposed binding table |

## Report

- Each row's answer, with what was measured, on both renderers where pixels matter.
- **Which of 1 to 5 is not feasible as proposed, and why.** For B1 above all: if the
  router cannot set the Tab starting point or take focus away, say so. Do not narrow the
  rule yourself. It returns to the owner.
- The proposed bindings (6).
- **Your estimate of Unit 1's size,** so the owner can decide 0.57 or 0.58.

**Breakage:** every prototype assertion has a row that can fail, shown failing. Two of
the last five handoffs specified a breakage that could not fail; check each first.

# Implementation handoff — RFC-125: a covered layer gets no pointer

**RFC.** `rfcs/done/125-a-page-under-a-dialog-still-answers-the-pointer.md`. Read the
finding, the rule, R-1 to R-4 and the Rulings.
**Release target.** 0.60.0, **first of its three items** (owner, Q-2). RFC-123 starts after
this lands, because both change `Inert`'s cover. The 0.60 housekeeping is independent
and may land before or after.
**Branch.** `rfc125-covered-pointer`, off `main`. Unit 0 is its own review. Land as `main`
requires:
1. rebase onto `main`;
2. push the rebased commits to a new branch name;
3. wait for CI green there;
4. fast-forward with `git push origin <sha>:main`.

## The rule

> **A covered layer gets no pointer.** Whatever the keyboard cannot reach (RFC-119), the
> pointer does not reach either: no hover, no pointer shape, no tooltip, no wheel. What
> covers it is a surface. A press on it (a click or a tap) and Escape close what is open
> (Law 7). Nothing else it receives does anything.

## Unit 0 — measure before changing anything

On `main`, both renderers, with pixel comparisons where the row says so. Start from
`menu_overlay.rs`'s `does_a_page_under_a_modal_dim_show_hover`, and keep its premise check
(the `RedrawRequested` before the frame).

| Scene | Measure |
|---|---|
| a dialog with `on_close_modals`, pointer on the dim, **one** wheel event | the messages published, and the status `UserInterface::update` returns |
| the same, **ten** wheel events | how many close messages |
| the same, with a sheet | as above |
| a dialog **without** `on_close_modals` | messages and status for a wheel on the dim |
| a page control under a dialog's dim | the pointer shape (`mouse_interaction`) over it |
| a page `tooltip` under a dialog's dim, pointer resting on its target | whether it is drawn (pixels) |
| a page under a **context menu** with its backdrop | hover (pixels), pointer shape, tooltip, and a wheel over the backdrop: does the page scroll? |
| a page under an **application `header_menu`** with its backdrop | as above |
| an iced `slider` dragged on the page, then a dialog opened by the application mid-drag, then the pointer moved and released | whether the slider moves while covered |

**Report each against the RFC's finding.** Items 2 and 3 of the finding are read from the
code, or inferred. If one is wrong, say so: the RFC is corrected before Unit 1.

**Also propose where R-2 lives.** The dim's `on_scroll` must go, since it publishes. Something
must then capture the wheel without publishing. Two candidates:
- **`OnEscape`**, which already wraps both backdrops and is snora's own;
- a capture in `Inert`'s cover.

Say which, and why. The constraint: a wheel over the **dialog's own card** must still
reach the dialog's own `scrollable`. Test that row in Unit 1.

## Unit 1 — the behaviour

### `Inert` (`crates/snora-interaction/src/inert.rs`, `inert/menu_cover.rs`)

- **`cursor_for_content` withholds the cursor whenever the layer is covered**
  (`covered(tree)`: the flag or a menu), not only when a menu covers it. It is used in both
  `update` and `mouse_interaction`, as now.
- **Rewrite the module doc's first section.** It says `Inert` *"leaves drawing, layout and
  the pointer alone: pointer blocking is the backdrops' job"*. That has not been true since
  0.59. State R-1.

### The backdrops (`crates/snora/src/render.rs`)

- `dim_backdrop` loses `on_scroll`. Its doc comment's *"scrolling over the dim is the same
  intent as clicking it, and either should dismiss"* goes with it. Replace it with R-2.
- `transparent_backdrop` captures the wheel likewise, without publishing.
- `dim_without_capture` is unchanged, but assert R-2 for it too.

## Tests

**Each row needs a breakage, checked first that it can fail** (named `cp` backups,
`sha256sum -c`; never `git checkout` or `restore`). Put them where Unit 0's measurements
live, or in a new `crates/snora/tests/covered_pointer.rs`. **Not in `render_semantics.rs`.**

| Assertion | Breakage |
|---|---|
| a dialog's dim, wheel: **no** message, status `Captured`, and the page does not scroll | restore `on_scroll` |
| ten wheel events on the dim: no message | — |
| a sheet's dim: the same | — |
| a dialog with no close message: wheel publishes nothing, status `Captured` | — |
| a wheel over the **dialog's card** scrolls the dialog's own `scrollable` | capture the wheel over the whole modal layer |
| a page under a dialog shows **no hover** (pixels), with the premise that it does with no dialog | `cursor_for_content` for menus only (0.59's code) |
| the pointer shape over a page control under a dialog is the default | the same |
| a page tooltip under a dialog is not drawn (pixels) | the same |
| a page under a context menu with its backdrop: no hover, no tooltip, no wheel | the same |
| a pointer resting on a page control when the dialog opens: no hover on the first frame under the dim | withhold the cursor in `update` only |
| a slider dragged mid-way when a dialog opens does not move while covered, and the release ends the drag | — |
| a click on the dim still publishes `on_close_modals` once; a press on a menu's backdrop, `on_close_menus` once | a capture that swallows presses too |
| a toast under no cover shows hover, and its × is pressable with a dialog open | withhold the cursor from the toast layer |
| an animation beneath a dialog keeps running (a redraw reaches it) | withhold window events |

**The existing tests pass unmodified:** `render_semantics.rs` (gate 5), `modal_keyboard.rs`,
`menu_overlay.rs` and `every_control.rs`. `does_a_page_under_a_modal_dim_show_hover`
measures rather than asserts. Turn it into an assertion of no hover, or remove it in
favour of the new row. Say which.

## Docs

**Rustdoc:** `Inert`'s module doc (above), `dim_backdrop` and `transparent_backdrop`, and
`AppLayout::on_close_modals` and `on_close_menus`. Each says it is published on a press or
on Escape, **not on a wheel**.

**CHANGELOG `[Unreleased]`:**
- **Fixed:** a wheel scroll over a dialog's or a sheet's dim closed it, once per wheel event.
- **Changed:** a page under a dialog, a sheet, or a menu with its backdrop shows no hover,
  no pointer shape and no tooltip, as a page under an open header dropdown already did.

**The book is the architect's:** Law 7, Law 8's pointer row, and the menus and dialogs guides.

## Acceptance criteria

1. Unit 0's table, reviewed, with the RFC corrected if any finding was wrong.
2. Unit 1: every row passing, and every breakage shown failing.
3. `render_semantics.rs` unmodified, and passing.
4. All of `ci.yaml`'s Rust quality gate commands (16), the Docs job's commands, the feature
   matrix, both rendered jobs, and parity. Then the branch CI.

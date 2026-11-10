# RFC-124 — A header dropdown's items cannot be pressed when its backdrop is up

**Status.** Proposed 2026-11-10. Awaiting the owner's rulings on Q-1 to Q-4.
**Raised.** 2026-11-10, architect. The dev team found the keyboard half while measuring
RFC-123 (`.git-exclude/review-request/123-unit0/`, 3.1). The architect measured the
pointer half (`.git-exclude/reviewed/123-unit0/`).
**Release target.** 0.59.0, before RFC-123's Unit 1, which builds on it.
**Priority.** **A defect in the documented wiring.** `menus.md` shows it, and it does not
work.

---

## The finding

`app_header` draws its dropdown **inside the header**, below the trigger. iced 0.14 offers
no absolute positioning at the overlay layer. To get a click-outside close, `menus.md`
tells applications to set `on_close_menus` and put an empty `Space` in
`AppLayout::header_menu`. The engine then lays a **full-window** backdrop over the
skeleton, header included.

**Measured, `snora::render`, an open `app_header` menu with one item:**

| | a click on the item | the item's keyboard reach |
|---|---|---|
| no backdrop | `MenuItemPressed` | a focus target |
| **the documented wiring** | **`Close` only**: the backdrop takes the click, and the item is not pressed | **none:** since 0.55 the skeleton is inert under a menu backdrop (RFC-119), and the dropdown is in the skeleton |

**The pointer half is the same at 0.54.0,** so it predates RFC-119. The documented wiring has
never let a dropdown item be clicked. The starter does not show it: its `File` menu has no
items, and its content goes in the `header_menu` layer.

**Who it reaches:** any application using `app_header` with menu items and the documented
backdrop. We do not know which adopters do. orbok has no header, and the starter avoids it.

## Proposal

**R-1. An open header menu covers the page below the header, not the header itself.**
- **The backdrop** lies over the skeleton's body, side bar and footer, not the header.
- **The inert region** under a header-menu backdrop is the same: the header stays
  reachable.
- **So the dropdown, the trigger and the header's end controls** stay live for the pointer
  and the keyboard alike. That keeps RFC-119's rule, "the keyboard reaches exactly what the
  pointer can press", with the header now on the reachable side.
- **A click below the header** closes the menu, as now. A click on the header's other
  controls does what they do: another trigger switches menus, as `MenuPressed` already
  says.
- **Escape** still closes the menu (Law 7).

**R-2. Context menus are unchanged.** Their content is in the `context_menu` layer, above
the backdrop, and works today.

**R-3. Law 8 and `menus.md` are rewritten to say R-1,** and the empty-`Space` idiom keeps
working unchanged: the slot still opts the backdrop in.

## Open questions, with suggested rulings

**Q-1 — R-1, or move the dropdown out of the header?** **Suggest R-1.**
- **The alternatives:**
  - draw the dropdown as an iced overlay of the header, so it floats above every layer;
  - or put it in the `header_menu` layer with computed positioning.
- **Each is a larger change,** and each meets RFC-119's `Inert`, which hides the overlays
  of a covered layer: the overlay would vanish whenever the header is covered.
- **R-1 changes only *where* the backdrop and the inert region begin.** The dropdown keeps
  its current look.

**Q-2 — how does the backdrop leave the header out?** **Suggest: the engine builds it
inside the skeleton,** as a layer over the region below the header. That is a `stack` of
the body region and the backdrop, rather than a full-window layer. The header's height is
then never needed. Unit 0 measures it against `render_semantics.rs`. **If gate 5's tests
must change, that is an escalation to the owner,** as the gate says.

**Q-3 — release.** **Suggest 0.59.0, before RFC-123's Unit 1.**

**Q-4 — correspondence.** **Suggest: the 0.58 → 0.59 guide states it in its first table.**
That covers what was broken, since when, and that nothing in the application changes. No
separate letter: we know of no adopter using `app_header`'s items with the backdrop. If
one tells us, it is answered.

## Review against snora's principles

- **Nobody is confused:** the documented wiring does what the guide says.
- **Safe:** the page beneath the open menu stays covered, so its controls cannot be pressed
  by mistake. The header is the menu's own row, and is not "beneath" it.
- **Robust:** a new test drives the documented wiring end to end, a click and a keyboard
  press on an item under `snora::render`. Nothing tested that until now: the existing
  dropdown test uses `render_menu` alone, without the engine.
- **Pointer and keyboard stay paired** (RFC-119). Both gain the header together.
- **ABDD:** layout and direction are unchanged.

## Acceptance criteria

1. **Unit 0:**
   - the backdrop and the inert region below the header;
   - a click and a keyboard press on a dropdown item under the documented wiring both
     publish `MenuItemPressed`;
   - a click below the header closes the menu;
   - Escape closes it;
   - `render_semantics.rs` unmodified, or an escalation.
2. **Unit 1:** the change, with a breakage row for each of the above, on both renderers.
3. **The book,** by the architect: Law 8, `menus.md`, and the 0.58 → 0.59 guide.

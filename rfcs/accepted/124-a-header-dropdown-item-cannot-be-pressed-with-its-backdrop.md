# RFC-124 — A header dropdown's items cannot be pressed when its backdrop is up

**Status.** Accepted 2026-11-10. Every open question ruled as suggested, as revised (see Rulings).
**Handoff.** [`handoffs/124-…`](../handoffs/124-a-header-dropdown-item-cannot-be-pressed-with-its-backdrop/implementation-handoff.md).
**Revised** 2026-11-10, before ruling. The owner asked for a review against the
fundamental philosophy, with "the initial cost is not the only factor and not the
strongest". The first draft chose the smallest change: leave the header live under its own
menu. It fixed the click, kept a page-shifting layout and a confusing opt-in idiom, and
rested on a premise that is false. This draft proposes the design a menu should have.
**Raised.** 2026-11-10, architect. The dev team found the keyboard half while measuring
RFC-123 (`.git-exclude/review-request/123-unit0/`, 3.1). The architect measured the
pointer half (`.git-exclude/reviewed/123-unit0/`).
**Release target.** Q-4. RFC-123's Unit 1 builds on this either way.
**Priority.** **A defect in the documented wiring.** `menus.md` shows it, and it does not
work.

---

## The finding: four problems with one cause

`app_header` draws its dropdown **inside the header**, as a `column` below the trigger.
`menus.md` explains why: *"iced 0.14's element tree does not expose absolute positioning at
the overlay layer"*. To get a click-outside close, it tells applications to set
`on_close_menus`, and to put an empty `Space` in `AppLayout::header_menu` so that the engine
installs a backdrop.

1. **A click on an item does not press it** (measured: `snora::render`, the documented
   wiring). The full-window backdrop lies over the header, takes the click, and publishes
   `Close`. **The same at 0.54.0,** so this predates RFC-119.
2. **The keyboard cannot reach the items** (measured by the dev team). Since 0.55, the
   skeleton is inert under a menu backdrop (RFC-119), and the dropdown is in the skeleton.
3. **Opening a menu pushes the whole page down.** The header's height is `Shrink`, so a
   `column` of items inside it makes the header taller while the menu is open. Every
   native menu floats over the page instead. (Read from `header.rs` and `menu.rs`; Unit 0
   measures it.)
4. **The opt-in idiom is confusing:** "put an empty `Space` in `header_menu`" means "a menu
   is open", and nothing in the name says so. An application that forgets it gets no
   backdrop. One that adds it gets problems 1 and 2.

**The common cause is a false premise.** iced 0.14 *does* expose positioning above the
page: `iced_core::overlay` is public (`iced_core-0.14.0/src/lib.rs:24,68`), as `Widget` is,
which RFC-116 relied on. iced's own `tooltip` and `pick_list` float their content that way,
anchored to the widget that opened it. The dropdown was put inline because of a constraint
that does not exist.

**The starter does not show any of this:** its `File` menu has no items, and its content
goes in the `header_menu` layer.

## Proposal: the design a menu should have

**R-1. The dropdown is an overlay anchored to its trigger,** drawn above the page, as every
native menu is:
- it opens below the trigger, aligned to its start edge (the end edge under RTL), and moves
  to stay inside the window;
- **it does not change the header's layout:** problem 3 is gone.

**R-2. While it is open, the menu is the top layer below any modal, and the rules already
written apply unchanged:**
- the whole page, header included, is covered for the pointer and the keyboard alike
  (RFC-119, Law 8);
- a click outside, the trigger included, closes it, as in every native menu;
- Escape closes it (Law 7);
- the trigger's focus is set aside, and comes back when the menu closes (RFC-119);
- the items are reachable and pressable, and RFC-123's arrows apply to them.

That fixes problems 1 and 2, by one rule rather than an exception.

**R-3. The engine learns that a header menu is open from the tree.** The menu marks its
open items, as RFC-123's prototype already does (`MenuPart`), so **the empty-`Space` idiom is
no longer needed**. That fixes problem 4: the application says which menu is open once, to
`app_header`, as it does today, and sets `on_close_menus` for the close message.
- **`AppLayout::header_menu` keeps its other use:** an application's own menu content in
  that layer, as the starter does.
- **Passing an empty `Space` there becomes unnecessary, and harmless.** The guide stops
  teaching it. Whether anything is deprecated is Q-2.
- **How the engine learns it at run time is Unit 0's to prove.** The backdrop and `Inert`
  are built in `view`, before the tree exists, so they would have to be always present and
  switched by what a survey finds, as `Inert` already is. **If that cannot be done
  cleanly, R-3 falls back to today's signal**, the `header_menu` slot, and Q-2 is asked
  again.

**R-4. Context menus are unchanged.** Their content is already a layer above the page.

## Open questions, with suggested rulings

**Q-1 — the anchored overlay (R-1, R-2), or keep the header live under its own menu (the
first draft)?** **Suggest the overlay.**
- **The first draft fixed problem 1 only** by making the header an exception to Law 8. A
  menu's own header would stay pressable under it, unlike any native menu.
- **It kept problems 3 and 4,** and it would have to be undone the day the dropdown moved
  where it belongs.
- **The overlay fixes all four,** with no exception to any rule. It costs more now, and that
  is not the deciding factor.

**Q-2 — the idiom: deprecate, or only stop teaching it?** **Suggest: stop teaching it, and
deprecate nothing.**
- `AppLayout::header_menu` stays, for menu content of the application's own.
- An empty `Space` passed to it is harmless under R-3: the backdrop is installed either way.
- Deprecating a *use* of a method cannot be expressed in Rust, and a warning on the method
  itself would wrong its legitimate use.
- The guide and the CHANGELOG say plainly that the idiom is no longer needed.

**Q-3 — placement inside RFC-119's `Inert`.**
- **The constraint:** while the page is covered, `Inert` hides the page's overlays
  (RFC-119, Unit 0 addition B: "nothing from a covered layer floats above what covers
  it"). The dropdown is the page's overlay, so as things stand it would be hidden.
- **Suggest:** the open menu's own overlay is not "from a covered layer": the covering
  backdrop belongs to it. `Inert` lets through the overlay marked as the open menu, and its
  focus targets, and only that.
- **Unit 0 proves this,** or measures the alternative: the engine hosts the dropdown
  overlay at the top level, anchored to the trigger's bounds. If neither holds without a
  new exception to Law 8, it returns to the owner.

**Q-4 — release.** **Suggest: decided at Unit 0's review.**
- Ship in 0.59 if Unit 0 shows the overlay is clean.
- Otherwise 0.60. 0.59 then ships the deprecation removal and the guide's known-defect note
  (already in the book), and RFC-123's Unit 1 waits for this.
- **Not the first draft as a stop-gap:** it would ship an exception to Law 8 only to remove
  it.

**Q-5 — correspondence.** **Suggest: the release's guide states the defect in its first
table:** what was broken, since when, and that an application changes nothing except, if
it likes, dropping the empty `Space`. No separate letter: we know of no adopter using
`app_header`'s items with the backdrop. If one tells us, it is answered.

## Review against snora's principles

- **Nobody is confused:**
  - the menu behaves like every menu: it floats over the page, closes on a click outside,
    and its items press;
  - the application says once which menu is open;
  - the guide stops teaching an idiom that needed a paragraph to explain.
- **Safe:**
  - the page under an open menu is covered for the pointer and the keyboard alike, with no
    exception;
  - nothing shifts under the reader's pointer when a menu opens.
- **Clean:**
  - one design, the one native menus have;
  - no exception to Law 8, and no special inert region;
  - one fewer thing for an application to remember.
- **Robust:**
  - the decisive test drives the documented wiring end to end under `snora::render`: a
    click and a key on an item, a click outside, Escape, and the header's height unchanged
    when the menu opens;
  - the existing dropdown test never ran under the engine, which is how this went unseen.
- **Built on what is true:** the premise that hid this was false. The RFC states it, and
  the book's "Why `header_menu` takes an empty `Space`" section is rewritten.
- **ABDD:** the dropdown is anchored to the trigger's start edge, mirrored under RTL, and
  kept inside the window.

## Acceptance criteria

1. **Unit 0:**
   - problem 3 measured: the header's height, closed and open;
   - a prototype of R-1 to R-3, with Q-3's placement proved (or the alternative measured);
   - under `snora::render` and the documented wiring, a click and a key on an item publish
     `MenuItemPressed`;
   - a click outside, and on the trigger, closes;
   - Escape closes, and focus returns to the trigger;
   - the header's height is unchanged when open;
   - RTL anchoring, and staying inside the window;
   - `render_semantics.rs` unmodified, or an escalation.
2. **Unit 1:** the change, with a breakage row for each of the above, on both renderers;
   RFC-123's Unit 1 follows.
3. **The book,** by the architect:
   - Law 8 (unchanged in substance; the header dropdown now under it);
   - `menus.md`: the wiring, and the "Why `header_menu` takes an empty `Space`" section
     corrected;
   - the release's guide.

## Rulings, 2026-11-10 (owner)

All approved, as revised:
- **Q-1:** the dropdown becomes an overlay anchored to its trigger. The first draft's
  header exception is rejected.
- **Q-2:** stop teaching the empty-`Space` idiom, and deprecate nothing.
- **Q-3:** the open menu's own overlay passes `Inert`, proved in Unit 0, or the top-level
  host measured instead. Anything that needs a new exception to Law 8 returns to the owner.
- **Q-4:** 0.59 or 0.60, decided at Unit 0's review. No stop-gap.
- **Q-5:** stated in the release's guide, with no letter.

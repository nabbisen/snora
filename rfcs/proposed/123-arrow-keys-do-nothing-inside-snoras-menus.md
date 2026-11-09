# RFC-123 — Arrow keys do nothing inside snora's menus

**Status.** Proposed 2026-11-09. Awaiting the owner's rulings on Q-1 to Q-5.
**Revised** 2026-11-09, before ruling, after the owner asked for a check against the
fundamental philosophy. R-3 became simpler: no record of how a menu was opened. The
"held arrow" claim was wrong, and is corrected. See "What the review changed".
**Raised.** 2026-11-09, architect, from RFC-116's long-term plan item **L-9**, found while
scoping 0.59. The owner accepted it into 0.59's scope (Q-1 of that scoping), menus only.
**Release target.** 0.59.0.
**Priority.** WCAG 2.1.1 is met through Tab. This is about what a keyboard user expects
in a menu: the owner's "nobody is confused".

---

## The finding

Since 0.55, a snora menu is keyboard-reachable:
- **every item is a focus target,** with a ring, and so is the header's trigger;
- **Tab and Shift+Tab** move between the items of an open menu, with the page beneath out
  of reach (RFC-119);
- **Enter and Space** press an item;
- **Escape** closes the menu, and focus returns to the trigger (RFC-119).

**What a keyboard user expects from a menu, and does not get:**
- **↓ and ↑ move between items.** Windows, GNOME, KDE and macOS menus all answer them, and
  so does the WAI-ARIA menu pattern. A keyboard user reaches for them first. In snora they
  do nothing.
- **Home and End** go to the first and last item.
- **Opening a menu from the keyboard puts focus on its first item.** In snora, the
  trigger's focus is set aside while the menu covers the page (RFC-119), and nothing is
  focused until the reader presses Tab.

That a menu ignores the keys every other menu answers is the confusion the owner's rule
forbids. Nothing is broken; the reader is simply left guessing.

**Scope:** snora's prefab menus, `render_menu` and `app_header`'s dropdown. A menu an
application builds itself and passes to `AppLayout::header_menu` or `context_menu` is
its own, though `snora::control` now gives its items focus.

## Proposal

**R-1. ↓ and ↑ move focus between the items of an open snora menu,** in their order,
skipping any item Tab skips. They are answered while focus is on an item of that menu, or
while nothing is focused (R-3), and captured, so nothing else sees them. **A held arrow
keeps moving,** as in every native menu: moving focus is navigation, and RFC-121's
"a held key presses once" governs pressing, not moving.

**R-2. Home and End** move focus to the first and last item.

**R-3. In an open menu where no item has focus, ↓ goes to the first item and ↑ to the
last.** Opening a menu never moves focus by itself, however it was opened:
- by the pointer;
- by Enter or Space on the trigger;
- by the application, for a context menu.

So from a focused trigger, **Enter then ↓** reaches the first item.

**R-4. Tab, Enter, Space and Escape are unchanged.**

**R-5. The rules join the keyboard map,** and `every_control.rs` gains the menu as a
subject for each.

## Open questions, with suggested rulings

**Q-1 — do ↓ and ↑ wrap at the ends?** **Suggest yes, on every platform:** ↓ on the last
item goes to the first. Windows and GNOME menus wrap; macOS menus stop; WAI-ARIA allows
either. One rule everywhere is simpler to state and to test than a per-platform rule. A
macOS reader who expects a stop meets a wrap, which costs a key press and can do no harm,
since moving activates nothing. Wrapping
means a reader can never get stuck at an end. It also matches Ctrl+Tab between tabs
(RFC-116 R-2). Tab, which moves *focus through the window*, still does not wrap; arrows
inside one menu do. The keyboard map states both, as it does for Tab and Ctrl+Tab.

**Q-2 — how does focus get into a menu? (revised)** **Suggest R-3 as written:** ↓ (or
↑) inside an open menu with nothing focused. No key on the trigger moves focus into the menu
by itself.
- **The first draft** moved focus to the first item when the menu was opened from the
  keyboard but not from the pointer. That needed a record of *how* the menu was opened.
- **It could not hold for context menus,** which the application opens. snora cannot know
  whether the application's own Shift+F10 or a right-click did it. Telling those apart
  would need a guess about the last input, the "focus-visible" heuristic RFC-121 rejected
  as unpredictable.
- **R-3 needs no history,** and works the same for header and context menus. Its cost is
  one key press (Enter, then ↓) where a native menu bar needs none.
- **A mouse user is untouched:** no ring appears until they press an arrow.

**Q-3 — does Tab inside an open menu change?** **Suggest no.** Native menus usually close
on Tab and move on. But in snora, a menu with its backdrop covers the page (RFC-119), so
there is nowhere for Tab to move on to until the menu closes. Today Tab moves between the
items, then the toasts, then back. Keeping that changes nothing that works, and Escape is
the way out, as for a dialog.

**Q-4 — type-ahead (pressing a letter jumps to the item that starts with it)?** **Suggest
not now.** It needs a timing rule and a matching rule across scripts, and that is a
separate design. It goes to RFC-116's long-term plan as future wanted.

**Q-5 — arrow keys in the tab bar?** **Not in this RFC (owner, scoping 0.59).** The tab
pattern uses a "roving" focus: Tab stops once at the tab bar, and the arrows move between
tabs. That changes Tab everywhere, and it is recorded as future wanted.

## Review against snora's principles

- **Nobody is confused:** a menu answers the keys every menu answers, and the rules are
  written down beside the rest.
- **Safe:**
  - **the arrows are answered only while focus is in the menu, or nowhere.** So a text
    field, or a page control under a menu without a backdrop, never loses them;
  - moving focus never presses anything, and Enter and Space still press once
    (RFC-121 R-3);
  - opening never moves focus, so nothing is focused that the reader did not move to.
- **What the review changed (2026-11-09):**
  - **R-3 no longer depends on how the menu was opened.** That rule could not hold for
    context menus without a heuristic RFC-121 had already rejected;
  - **the first draft claimed a held arrow moves once per press.** That misapplied
    RFC-121's rule, which is about pressing a control. A held arrow keeps moving, as in
    every menu, and moving activates nothing.
- **Clean:** no new public API. The behaviour belongs to snora's own menus.
- **Pointer users are untouched:** opening with the pointer moves no focus and draws no
  ring (R-3, RFC-121).
- **ABDD:** a menu is a vertical list, so ↓ and ↑ do not mirror under RTL. Home and End
  are first and last in reading order.

## What this is not

- Not type-ahead (Q-4), and not the tab bar (Q-5).
- Not menus the application builds itself.

## Acceptance criteria

1. **Unit 0:** a prototype that measures R-1 to R-3 in `render_menu`, `app_header`'s
   dropdown and a context menu:
   - wrap;
   - an item Tab skips, skipped;
   - ↓ and ↑ from nothing focused;
   - opening by pointer, by key and by the application, each moving no focus;
   - the arrows captured, and not seen by the application;
   - a focused text field, or a page control under a menu without a backdrop, keeping its
     arrows;
   - a held arrow moving repeatedly.
2. **Unit 1:** the behaviour, a test and a breakage row per rule, and `every_control.rs`
   covering the menu.
3. **The book,** by the architect: the keyboard map and the menus guide.

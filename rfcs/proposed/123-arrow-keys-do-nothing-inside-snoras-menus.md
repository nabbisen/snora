# RFC-123 — Arrow keys do nothing inside snora's menus

**Status.** Proposed 2026-11-09. Awaiting the owner's rulings on Q-1 to Q-5.
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
skipping disabled items as Tab does. The keys are answered only while focus is on an item
of that menu, and captured, so nothing else sees them.

**R-2. Home and End** move focus to the first and last item.

**R-3. Opening a menu from the keyboard moves focus to its first item.** Opening it with the
pointer does not. That matches RFC-121: a click presses and does not focus, so a mouse user
never sees a ring they did not ask for. The trigger knows which opened it (a key press or
a pointer release), and says so.

**R-4. Tab, Enter, Space and Escape are unchanged.**

**R-5. The rules join the keyboard map,** and `every_control.rs` gains the menu as a
subject for each.

## Open questions, with suggested rulings

**Q-1 — do ↓ and ↑ wrap at the ends?** **Suggest yes:** ↓ on the last item goes to the
first. Windows and GNOME menus wrap; macOS menus stop. WAI-ARIA allows either. Wrapping
means a reader can never get stuck at an end. It also matches Ctrl+Tab between tabs
(RFC-116 R-2). Tab, which moves *focus through the window*, still does not wrap; arrows
inside one menu do. The keyboard map states both, as it does for Tab and Ctrl+Tab.

**Q-2 — which keys open a menu from its trigger?** **Suggest Enter, Space, and ↓,** all
three landing on the first item. Enter and Space press the trigger as they press any
control. ↓ is what a keyboard user presses on a menu-bar entry. **↑ opens it on the last
item,** as WAI-ARIA describes.

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
  - the arrows are captured only while focus is in the menu, so a text field or the page
    never loses them;
  - a held arrow moves once per press, by the held-key rule (RFC-121 R-3);
  - focus on open lands on an item, which does nothing until Enter.
- **Clean:** no new public API. The behaviour belongs to snora's own menus.
- **Pointer users are untouched:** opening with the pointer moves no focus and draws no
  ring (R-3, RFC-121).
- **ABDD:** a menu is a vertical list, so ↓ and ↑ do not mirror under RTL. Home and End
  are first and last in reading order.

## What this is not

- Not type-ahead (Q-4), and not the tab bar (Q-5).
- Not menus the application builds itself.

## Acceptance criteria

1. **Unit 0:** a prototype that measures R-1 to R-3 in `render_menu` and `app_header`'s
   dropdown:
   - wrap;
   - a disabled item skipped;
   - keyboard-open versus pointer-open;
   - ↑ opening on the last item;
   - the arrows captured, and not seen by the application;
   - a focused text field elsewhere unaffected.
2. **Unit 1:** the behaviour, a test and a breakage row per rule, and `every_control.rs`
   covering the menu.
3. **The book,** by the architect: the keyboard map and the menus guide.

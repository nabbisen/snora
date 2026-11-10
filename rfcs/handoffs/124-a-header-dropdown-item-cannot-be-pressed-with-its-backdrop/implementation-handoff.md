# Implementation handoff — RFC-124: the header dropdown becomes an anchored overlay

**RFC.** `rfcs/accepted/124-a-header-dropdown-item-cannot-be-pressed-with-its-backdrop.md`.
Read the finding (four problems, one false premise), R-1 to R-4, and the Rulings.
**Release target.** 0.59 or 0.60, **decided at Unit 0's review** (Q-4). No stop-gap.
**Order.** Before RFC-123's Unit 1, which builds its header scenes on this. The 0.59
deprecation removal (`housekeeping-0-59`) is independent.
**Branch.** `rfc124-menu-overlay`, off `main`. Land as `main` requires:
1. rebase onto `main`;
2. push the rebased commits to a new branch name;
3. wait for CI green there;
4. fast-forward with `git push origin <sha>:main`.

## What is being made

- **R-1:** the dropdown is an overlay anchored to its trigger, drawn above the page. It
  opens below the trigger, at its start edge (the end edge under RTL), stays inside the
  window, and **does not change the header's layout.**
- **R-2:** while it is open, it is the top layer below any modal:
  - the whole page, header included, is covered for the pointer and the keyboard;
  - a click outside, the trigger included, closes it;
  - Escape closes it;
  - the trigger's focus is set aside and comes back;
  - the items are reachable and pressable.
- **R-3:** the engine learns that a header menu is open **from the tree** (the menu marks
  its open items, as RFC-123's prototype `MenuPart` does), so the empty `Space` in
  `header_menu` is no longer needed. `header_menu` keeps its use for an application's own
  menu content.

## Unit 0: prove it on a prototype

| # | Measure |
|---|---|
| 1 | **Problem 3:** the header's height with the menu closed and open, today, under `snora::render`. Report the shift in pixels |
| 2 | **The overlay:** a prototype dropdown as an `iced_core::overlay` anchored to the trigger. Below it, start-aligned; mirrored under RTL; moved to stay inside the window when the trigger is near an edge |
| 3 | **Q-3, the placement.** While the page is covered, `Inert` hides the page's overlays (RFC-119). **Prove** that letting through only the overlay marked as the open menu, and its focus targets, keeps every RFC-119 test passing unmodified. **Or measure the alternative:** the engine hosts the dropdown at the top level, anchored to the trigger's bounds. Report which is cleaner, and why |
| 4 | **R-3, at run time.** The backdrop and `Inert` are built in `view`, before the tree exists. Can they be always present and switched by what a survey finds, as `Inert` already is? **If not cleanly, say so:** R-3 then falls back to the `header_menu` signal, and Q-2 goes back to the owner |
| 5 | **End to end, the documented wiring, `snora::render`:** a click and a key (Enter after Tab or ↓) on an item publish `MenuItemPressed`; a click outside, and a click on the trigger, close; Escape closes, and focus returns to the trigger; the header's height is unchanged while open |
| 6 | **Both wirings work:** `menus.md`'s (with the empty `Space`) and the starter's (content in `header_menu`). The empty `Space` must be harmless |
| 7 | **Gate 5:** `render_semantics.rs` unmodified. If it must change, stop: that is an escalation to the owner |

Both renderers. Every measurement that is an assertion gets a breakage row, checked that it
can fail. Named `cp` backups, ending with `sha256sum -c`.

## Report

- The table.
- Q-3's answer.
- Whether R-3 holds at run time.
- **Your estimate of Unit 1's size, so the owner can rule on 0.59 or 0.60.**

## Unit 1, after the review

The change itself, with the tests above as the production tests, and a breakage row each.
The book is the architect's:
- Law 8;
- `menus.md`: the wiring, and the "Why `header_menu` takes an empty `Space`" section;
- the release's guide.

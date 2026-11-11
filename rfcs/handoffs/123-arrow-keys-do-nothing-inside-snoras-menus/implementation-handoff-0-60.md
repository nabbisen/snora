# Implementation handoff — RFC-123 for 0.60: arrow keys, on the anchored dropdown

**RFC.** `rfcs/done/123-arrow-keys-do-nothing-inside-snoras-menus.md`. Read R-1 to R-5,
the revision note and the Rulings.
**This replaces** the first handoff's Unit 1
([`implementation-handoff.md`](implementation-handoff.md)). Its rules, and Unit 0's review
(`.git-exclude/reviewed/123-unit0/`, rulings 3.2 to 3.5), still stand. What changed: 0.59
made the header dropdown an overlay anchored to its trigger (RFC-124). The prototype's
placement, `MenuKeys` over the layer stack using `operate`, cannot see into an overlay.
**Release target.** 0.60.0, **after RFC-125 lands** (owner, Q-2): both change `Inert`'s
cover. **If Unit 0 shows that rule 4 cannot hold as written, stop and report.** It returns to
the owner before anything ships (owner, Q-2).
**Branch.** `rfc123-menu-arrows-2`, off `main` after RFC-125. The prototype `5dc7140` on
`rfc123-menu-arrows` may be reused. Unit 0 is its own review. Land as `main` requires:
1. rebase onto `main`;
2. push the rebased commits to a new branch name;
3. wait for CI green there;
4. fast-forward with `git push origin <sha>:main`.

## The rules (unchanged, with 3.2 folded in)

1. **↓ and ↑ move focus between the items of an open snora menu,** in order, skipping any
   item Tab skips, and **wrap at the ends.**
2. **Home and End** go to the first and last item.
3. **With nothing focused in an open menu, ↓ starts at the first item and ↑ at the last.**
   Opening a menu never moves focus by itself.
4. **The arrows (and Home and End) are answered only while focus is on an item or the trigger
   of that menu, or while nothing in the window is focused,** and they are captured. A
   focused text field, or a page control under a menu without a backdrop, keeps its arrows.
5. **A held arrow keeps moving.** Moving never presses.
6. **Tab, Enter, Space and Escape are unchanged.**
7. Menus are vertical lists, so the arrows do not mirror under RTL.
8. **The plain keys only** (3.4): Shift+↓ and Ctrl+Home are left alone.

**Scope:** `render_menu` (a context menu) and `app_header`'s dropdown. Not menus the
application builds itself.

## What 0.59 changed, as facts to build on

- **The dropdown is an overlay** of `MenuAnchor` (`crates/snora-interaction/src/menu_anchor.rs`).
  The runtime offers an overlay every event **before** the widget tree, and every
  operation run through `UserInterface::operate` (a Tab is one). **A widget's own
  `operate` over its content does not reach it** (RFC-124 Unit 0, 3.3).
- **A context menu is still inline,** a layer of the engine's `stack`, wrapped in `Inert`
  (`crates/snora/src/render.rs:264`).
- **The dropdown scrolls an item into view when it takes focus,** on the focus operation.
  An arrow move must do the same.
- **With a close message, an open dropdown covers the page, header and trigger included.**
  The trigger's focus is set aside while the menu is open, and comes back on Escape
  (RFC-119, RFC-124). So 3.2's "focus on the trigger" arises **only without a close
  message**, where the page stays live. With one, the only focus targets outside the
  menu are the toasts.
- **After RFC-125,** nothing covered gets the pointer. This RFC changes no pointer
  behaviour.

## The design constraint

Rule 4 needs **one view of the window**: whether focus is on this menu's item or trigger,
somewhere else (a text field, a toast, a page control), or nowhere. The two hosts see
different parts:
- the dropdown overlay is offered the key first, and sees only its own items;
- a widget over the stack sees the base tree (the toasts, the page, a context menu), and
  not the overlay.

**Bound the solution:**
- **No new public API, and no new application wiring.** An application that renders through
  `snora::render` gets the arrows, as it gets Escape. (Tab still needs `traverse`. The arrows
  must not.)
- **No stale view.** "Nothing is focused" must be true at the key press, not as of an earlier
  frame. An application can move focus with a task between two events.
- **One mechanism for both hosts** (RFC-124 Unit 0, 3.3), unless Unit 0 shows why two are
  needed.

## Unit 0 — propose and measure one mechanism

**A candidate, to try first and measure, not a ruling.** A widget over the whole layer stack,
as the prototype was, which also takes charge of its content's overlay. It wraps the
overlay returned by its content's `overlay()`, so it is offered every key **first** and
can see both parts in one place:
- the base tree, through its content's `operate`;
- the overlay, through the wrapped overlay's `operate`.

If that cannot hold, propose another, and say why the candidate failed.

**Measure, on both renderers:**

| Scene | Measure |
|---|---|
| header dropdown, with a close message: ↓ / ↑ / Home / End, wrap at both ends | rules 1, 2 |
| the same, nothing focused after opening by pointer and by Enter on the trigger | rule 3: first or last item; **no focus before the key** |
| the same, a toast's × focused, then ↓ | rule 4: the toast keeps it, the menu does not move |
| header dropdown **without** a close message, the trigger focused, ↓ | 3.2: the first item |
| the same, a page `text_input` focused, ↓ | rule 4: the field keeps it |
| the same, a page control focused, ↓ | rule 4: nothing moves in the menu |
| the application focuses a page text field **with a task** while the menu is open, then ↓ | no stale view: the field keeps it |
| a context menu, opened by the application, ↓ with nothing focused | rule 3 |
| a 40-item dropdown in 600 px: End, then ↓ (wrapping to the first) | each item focused is **in view, by pixels** |
| a held ↓ (`repeat: true`) | moves each time |
| Enter after an arrow | presses once, publishing what a click publishes |
| the keys while answered | status `Captured`; the application's `keyboard::listen()` does not receive them |
| Shift+↓, Ctrl+Home | not answered |
| a dialog open over an open menu | silent (as measured in Unit 0) |
| the base tree is offered a key that the overlay captured | whether it is, since iced's order decides whether a key could be answered twice |

**Report what does not fit,** as the first Unit 0 did.

## Unit 1

The behaviour, with a test per rule and a breakage row each, checked first that it can fail
(named `cp` backups, `sha256sum -c`; never `git checkout` or `restore`). Compare **pixels** for
the ring and for scroll-into-view, as RFC-124's units did.
- `every_control.rs` takes the menu as a subject for the arrow rules, on the runtime rig.
- `render_semantics.rs` is unmodified.
- **Rustdoc:** wherever the keys are answered. Say that `render_menu` used outside
  `snora::render` gets no arrows (Unit 0 review).
- **CHANGELOG `[Unreleased]`, Added:** arrow keys, Home and End in snora's menus (RFC-123).
- **The book is the architect's:** the keyboard map and the menus guide.

## Acceptance criteria

1. Unit 0's table and mechanism, reviewed.
2. Unit 1: every breakage row shown failing.
3. `render_semantics.rs` unmodified, and passing.
4. All of `ci.yaml`'s Rust quality gate commands (16), the Docs job's commands, the feature
   matrix, both rendered jobs, and parity. Then the branch CI.

# RFC-116 — No snora control can be reached from the keyboard

**Status.** Proposed.
**Raised.** 2026-11-04, architect, from the finding behind RFC-114's correction
(`.git-exclude/reviewed/iced-buttons-take-no-keyboard-input/`).
**Owner's direction, 2026-11-04:** *"Keyboard compatibility is required. Even if
immediate action is not possible, it must be managed as a long-term plan."*
**Release target.** R-1 in 0.55.0 (Q-4). R-2 is a plan with triggers, not a release.
**Priority.** WCAG 2.1.1 (Keyboard) is level A, the lowest level. snora's own controls
do not meet it, and until 2026-11-04 the book said they did.

---

## The finding

**iced 0.14's button takes no keyboard input.** `update` handles mouse and touch
only, and `operate` registers no focus target. Only `text_input` and `text_editor`
implement `Focusable`. A probe confirmed it:
- Tab, Enter and Space, three times each, pressed nothing;
- a click pressed the button, and typing reached a text input.

**So every control snora draws is pointer and touch only:**
- sidebar items, tabs and breadcrumbs;
- toast and notice dismiss buttons, and the chip ×;
- menu items, and every `design::button` variant.

The keyboard reaches only what an application wires itself:
- its own shortcuts (orbok switches views with `Ctrl/Cmd+1..6`);
- `snora::keyboard::dismiss_on_escape`, which closes overlays;
- `snora::keyboard::cycle_zones`, which decides the next zone. Inside a zone it can
  move focus only between text inputs, the only widgets iced 0.14 can focus.

**Why snora cannot simply make its buttons focusable.** A focusable control is a
custom widget implementing iced's `Widget` trait, which iced exposes only under its
`advanced` feature (`iced-0.14.0/src/lib.rs:524`). The owner has ruled that `advanced`
must never be a default (RFC-078). Moving focus (`focus_next`) needs no `advanced`,
but it only reaches widgets that are already focusable.

## Proposal

**R-1 — now: a keyboard route to every snora navigation control, with no `advanced`.**
Each is a pure function shaped like `cycle_zones` and `dismiss_on_escape`. It takes a
key and modifiers, and returns `Option<Message>`, or the existing action type the
control already emits (`TabAction`, `BreadcrumbAction`). snora installs no
subscription; the application wires `iced::keyboard::listen()`. That is the existing
non-capture policy.

| Control | Helper | Suggested binding (Q-1) |
|---|---|---|
| Sidebar item *n* | `side_bar_item(key, mods, &SideBar)` → that item's `on_press` | `Ctrl`/`Cmd` + `1`…`9` |
| Next or previous tab | `tab_step(key, mods, &TabBar<Id>)` → `Option<TabAction<Id>>` for the neighbour | `Ctrl+Tab` / `Ctrl+Shift+Tab` |
| Breadcrumb parent | `crumb_up(key, mods, &[Crumb<Id>])` → `Option<BreadcrumbAction<Id>>` for the parent | `Alt+↑` |
| Newest toast | `dismiss_on_escape` also dismisses the newest toast, once no overlay is open (Q-2) | `Escape` |

- **Every binding is in one keyboard map,** a new book page, and in each control's
  rustdoc.
- **Every example that draws a control wires its helper,** so copying an example gives
  an application the keyboard route as well (Q-3).

**R-2 — long term: real focus, managed as a 1.0 gate with triggers.** A new 1.0 gate:
*"Every snora control is reachable and operable from the keyboard (WCAG 2.1.1),
without a helper."* It starts ⬜. Two routes can close it, and each has a trigger a
reader can predict:
- **(a) upstream.** At **every iced upgrade**, re-run the keyboard probe. That is a
  planned, mechanical event, and gate 1's row is what an upgrade reads, so the
  instruction goes there. If iced's button becomes focusable and operable, snora
  adopts it in that upgrade and the gate closes.
- **(b) snora's own.** snora's controls become focusable custom widgets, behind an
  **opt-in** feature that enables `advanced`. RFC-078's ruling allows this, because
  the feature is not a default. **Trigger:** when the 1.0 register is next swept, if
  (a) has not closed the gate and no iced release is in sight, measure (b)'s cost (RFC-078's
  method) and decide.

**Neither route is fixed in advance.** What is fixed: the gate exists, it is checked
at every iced upgrade, and it blocks 1.0.

**R-3 — say it plainly, in the book.** `accessibility.md` states WCAG 2.1.1's status:
not met by snora's controls; R-1's routes are what exists; R-2 is the plan. The
withdrawn Enter/Space claim is already corrected (`0c6e59e`).

## Open questions

**Q-1 — the bindings.** **Suggest the table above.** They follow common desktop
conventions where one exists: `Ctrl+Tab` for tabs, `Alt+↑` for "up a level", and
`Ctrl/Cmd+digit`, which orbok already uses. An application can bind others, since
each helper's key test is separate from its decision. **Collisions** with an
application's own shortcuts are the application's to resolve. The keyboard map says
so.

**Q-2 — Escape and toasts.** Escape closes modals, then menus, today. **Suggest:**
once neither is open, Escape dismisses the newest toast. The order is predictable
(the most recent, innermost thing first). It gives a keyboard user the only way to
clear a persistent error, which RFC-113 makes the default.

**Q-3 — ABDD and opt-in wiring.** R-1 still needs the application to wire it, which
is accessible by choice, the trap RFC-113 met. A deprecation cannot steer here, since
nothing existing is superseded. **Suggest:**
- the examples and the getting-started guide wire every helper;
- the keyboard map heads the accessibility guide.

That is as close to "by default" as snora can get without capturing keys, which its
non-capture policy rules out. The same limit is stated in R-3.

**Q-4 — release.** **Suggest 0.55.0.** 0.54.0 already carries RFC-112 and Theme E's
three RFCs, and R-1 adds public API in four places, which deserves its own review.
R-2's gate row and R-3's statement go in now, as docs, because they are the plan the
owner asked for.

**Q-5 — the `cycle_zones` doc.** It moves "focus" between zones, but inside the
sidebar, tab bar and header there is nothing focusable. **Suggest:** its rustdoc and
the keyboard map say which zones can hold focus (those with text inputs) and which
cannot.

## Review against snora's principles

- **ABDD:** R-1 is the reachable part now. R-2 makes the rest a 1.0 blocker rather than
  a hope. Q-3 states where "by default" stops, and why.
- **Rules a reader can predict:** conventional bindings; Escape takes the innermost
  thing first; R-2's triggers are planned events (an iced upgrade, a register sweep),
  never unplanned ones.
- **Verify claims:** the button finding rests on source and a probe with positive
  controls. R-2(a)'s check reuses that probe.
- **Correspondence cost:** no letter. The book carries the plan and the corrected
  claim.

## What this is not

- Not enabling `advanced` by default.
- Not snora capturing keys.
- Not keyboard routes inside application content (dialog bodies and sheets), which
  the application owns.

## Acceptance criteria

1. R-1: each helper tested at the key level, with each assertion shown failing. Every
   example that draws the control wires it.
2. R-2: the gate row in `api-freeze-review.md`, and the probe instruction in gate 1's
   row. The architect writes both on acceptance.
3. R-3 and Q-5: the docs, traced to the probe.

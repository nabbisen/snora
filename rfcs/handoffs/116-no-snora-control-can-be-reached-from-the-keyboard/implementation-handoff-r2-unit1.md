# Implementation handoff — RFC-116 R-2, Unit 1: the shortcuts router, and the Escape field

**RFC.** `rfcs/done/116-…`, **"Rulings, 2026-11-08: R-2, after Unit 0"** (confirmed by
the owner). Read it first: it holds the rules, the bindings table, R2-1 to R2-3 and L-8.
**Review.** `.git-exclude/reviewed/116-r2-unit0/review-result.md`, including its
refinements.
**Release target.** 0.57.0.
**Branch.** `rfc116-r2-unit1`, off `main`. Land as `main` requires. Unit 0's prototypes stay
on their branch: replace them, do not move them.

## 1. `Shortcuts` and `AppLayout::shortcuts`

- **The builder is built in `view`,** from the same data the application hands its
  widgets:
  - `.side_bar(&side_bar)`;
  - `.tabs(&tab_bar, &on_action)`;
  - `.crumbs(&crumbs, &on_action)`;
  - **`.digits(Digits::Tabs | Digits::SideBar)`.** One call, one enum, so the digits
    cannot be given to both. With no `.digits(…)`, no digit route.
- **Every route is opt-in.** A route the builder does not name answers nothing.
- **Placement:** the engine wraps the whole skeleton in the router, **inside** the
  skeleton's `Inert`. The router delivers to its content first, and answers only if
  `!shell.is_event_captured()`.

## 2. The rules, each a test with a breakage row

| Rule | Test | Breakage |
|---|---|---|
| silent under every overlay that blocks the pointer; answers beneath a menu with no backdrop | RFC-119's table | the router outside `Inert` |
| captured when it answers | the status | publish without capturing |
| **a shortcut does what a click does:** the same message, focus taken away, the Tab starting point on the target; a disabled or absent target does nothing | per route; the view swap (focus in view A, then `Ctrl+2`): nothing focused, and Tab lands after the target | drop the focus step; drop the starting point |
| a focused control that uses the key wins | `text_editor` + `Alt+↑` | answer before the content |
| **a focused text field takes every arrow key, with any modifier** | `text_input` focused: `Alt+↑` and `Cmd+↑` fire no parent route | remove the text-field check. The router asks the focused target whether it is a text field: iced's `Operation::text_input`, reported by the same widget as its `focusable` (`iced_core-0.14.0/src/widget/operation.rs:53`) |
| **digits match the physical digit row** | an AZERTY-shaped event (logical `"&"`, physical `Digit1`, Ctrl) reaches destination 1 | match the logical key |
| named keys match the logical key | `Ctrl+Tab`, `Ctrl+PgDn`, `Alt+↑` | — |
| `9` is the last; a digit beyond the count does nothing | four tabs: `Ctrl+9` goes to the 4th; `Ctrl+5` does nothing | treat every digit beyond the count as last |
| no target in this view: nothing | digits set to tabs, in a view with no tab bar | — |
| every binding has a modifier | a plain `1`, an arrow or `Tab` fires no route | — |
| RTL | tab and crumb order mirror: next tab is the one Tab order calls next | — |

## 3. Targets by tag, not by count

Each snora widget **tags its own controls as it builds them:** role (sidebar item, tab,
crumb) and index, read through `operation.custom`, as `start_here` is. The router finds
"tab 3" by its tag. That is robust if a control is ever disabled, and it makes Unit 0's
region-id lookup and the extra markers unnecessary.

## 4. The Escape field

- **`AppLayout` gains an escape field with named fields.** Its exact name is your
  proposal; follow `Escape { modal, menu }` from the amendment.
- **The rule:** Escape publishes the overlay's escape message if one is set, and otherwise
  its click-outside message. The click-outside message still answers clicks.
- **Placement:** on the overlay's own layer, as `OnEscape` is.
- **Tests:** Unit 0's four-row table, for a dialog, a sheet and a menu, with a breakage
  row.
- **`snora::keyboard::dismiss_on_escape` is deprecated:** `since = "0.57.0"`, its note
  naming the field and "removed in 0.60.0".

## 5. `traverse` claims plain Tab and Shift+Tab only

Today `snora::keyboard::traverse` claims Tab with any modifier, so `Ctrl+Tab` moves focus.
With `Ctrl+Tab` bound to the next tab, every time the router is silent, `Ctrl+Tab` would
reach `traverse` and move focus instead.
- **Fix:** no Ctrl, Alt or Command.
- **Test:** `Ctrl+Tab` and `Alt+Tab` return `None`.
- **Breakage:** ignore the modifiers.
- **CHANGELOG:** Fixed.

## 6. Docs

- **Rustdoc:** `Shortcuts`, every route and its binding, the rules above, and the macOS
  note: *the macOS bindings follow Apple's conventions but have not been checked on a
  Mac* (L-8).
- **CHANGELOG `[Unreleased]`:**
  - Added: `Shortcuts`, `AppLayout::shortcuts`, the escape field;
  - Deprecated: `dismiss_on_escape`, removed in 0.60.0;
  - Fixed: `traverse`'s modifiers.
- **The book's keyboard map page is the architect's.** Report the final API names.

## Acceptance criteria

1. Every row of section 2, section 4's table and section 5, each with its breakage row
   shown failing (named `cp` backups, `sha256sum -c`). Check each row can fail before
   relying on it.
2. `render_semantics.rs` unmodified. Unit 0's measurements still hold in the production
   router.
3. All of `ci.yaml`'s Rust quality gate commands, the Docs job's commands, the feature
   matrix, the opt-out probe, both rendered jobs and parity. Then the branch CI.

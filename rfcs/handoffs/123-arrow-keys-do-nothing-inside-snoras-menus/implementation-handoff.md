# Implementation handoff — RFC-123: arrow keys inside snora's menus

> **Superseded for Unit 1 (2026-11-11):** see [`implementation-handoff-0-60.md`](implementation-handoff-0-60.md),
> re-planned on RFC-124's anchored dropdown. Unit 0 below is done and reviewed.

**RFC.** `rfcs/done/123-arrow-keys-do-nothing-inside-snoras-menus.md`. Read R-1 to R-5,
the revision note and the Rulings.
**Release target.** 0.59.0, **after RFC-124** (owner, 2026-11-10): Unit 1's header scenes are
built on RFC-124's anchored dropdown. Unit 0's review (`.git-exclude/reviewed/123-unit0/`)
settled 3.2 to 3.5. **Unit 0 first,** as its own review, then Unit 1. If Unit 0 is
small and clean, the review may let Unit 1 follow on the same branch.
**Branch.** `rfc123-menu-arrows`, off `main`. Land as `main` requires:
1. rebase onto `main`;
2. push the rebased commits to a new branch name;
3. wait for CI green there;
4. fast-forward with `git push origin <sha>:main`.

## The rules

1. **↓ and ↑ move focus between the items of an open snora menu,** in order, skipping any
   item Tab skips, and **wrap at the ends.**
2. **Home and End** go to the first and last item.
3. **With nothing focused in an open menu, ↓ starts at the first item and ↑ at the last.**
   Opening a menu never moves focus by itself: not by pointer, not by Enter or Space on the
   trigger, not when the application opens a context menu.
4. **The arrows (and Home and End) are answered only while focus is on an item of that
   menu, or while nothing is focused,** and captured. A focused text field, or a page
   control under a menu without a backdrop, keeps its arrows. These keys arrive only after
   the focused content has had its chance, as RFC-116 R-2's router does.
5. **A held arrow keeps moving.** Moving never presses: Enter and Space still press once.
6. **Tab, Enter, Space and Escape are unchanged.**
7. Menus are vertical lists, so the arrows do not mirror under RTL.

**Scope:** `render_menu`, `app_header`'s dropdown, and a context menu built from
`render_menu`. Not menus the application builds itself.

## Unit 0: measure on a prototype

For the header dropdown and a context menu, on both renderers:

| Measure | Expected |
|---|---|
| ↓ / ↑ between items, and wrapping at each end | rules 1 |
| Home / End | rule 2 |
| ↓ and ↑ with nothing focused, after opening by pointer, by Enter on the trigger, and by the application | rule 3: the first item / the last; and **no focus at all before the key** |
| the keys captured, and the application's `keyboard::listen()` not receiving them | rule 4 |
| a focused `text_input` elsewhere (a menu without a backdrop), then ↓ | the field keeps it; the menu does not move |
| a held ↓ (`repeat: true`) | moves each time |
| Enter after reaching an item by arrows | presses it once, publishing what a click publishes |

**Report what does not fit.** For example: if snora's menus never have an item that Tab
skips, say so, and that clause has nothing to test. **Where these keys are answered** (in
the menu's own widget, or in the overlay layer) is your proposal: say which, and why.

## Unit 1

The behaviour, with a test per rule and a breakage row each. Check each row can fail first
(named `cp` backups, `sha256sum -c`).
- `every_control.rs` takes the menu as a subject for the arrow rules.
- `render_semantics.rs` is unmodified.
- **CHANGELOG `[Unreleased]`, Added:** arrow keys in snora's menus (RFC-123).
- **The book is the architect's:** the keyboard map and the menus guide.

## Acceptance criteria

1. Unit 0's table, reviewed.
2. Unit 1: every breakage row shown failing.
3. All of `ci.yaml`'s Rust quality gate commands, the Docs job's commands, the feature
   matrix and both rendered jobs, with parity. Then the branch CI.
